use std::error::Error;
use std::io;
use std::io::{SeekFrom};
use std::ops::{ Range};
use log::error;
use thiserror::Error;

pub type PeekFrom = SeekFrom;
#[derive(Debug, Error)]
pub enum AnalysisError {
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error("Invalid seek to {0}.")]
    InvalidSeek(usize),
    #[error("Range {0}..{1} is out of bounds.")]
    InvalidRange(usize, usize)
}

pub trait Analyser<T: Sized + PartialEq + Clone> {
    type E: Error + From<AnalysisError>;
    #[inline]
    fn contents(&self) -> &Vec<T>;

    #[inline]
    fn pos(&self) -> usize;

    #[inline]
    fn set_cursor(&mut self, cursor: usize);

    #[inline]
    fn len(&self) -> usize { self.contents().len() }

    #[inline]
    fn peek(&self) -> Option<&T> { self.contents().get(self.pos()) }

    #[inline]
    fn is_end(&self) -> bool { self.pos() >= self.len() }

    fn seek(&mut self, pos: SeekFrom) -> Result<usize, Self::E> where Self: Sized  {
        let position = calculate_peek_position(self, pos)?;

        if position < 0 { return Err(AnalysisError::InvalidSeek(position).into()); }
        self.set_cursor(position);
        Ok(self.pos())
    }

    fn get(&mut self) -> Option<T> {
        let current = self.pos();
        match self.contents().get(current).cloned() {
            Some(element) => {
                self.set_cursor(current + 1);
                Some(element)
            },
            None => None,
        }
    }

    fn get_to(&mut self, pos: SeekFrom) -> Result<Option<T>, Self::E> where Self: Sized {
        let position = calculate_peek_position(self, pos)? - 1;

        if position < 0 { return Err(AnalysisError::InvalidSeek(position).into()); }

        self.set_cursor(position);
        Ok(self.get())
    }

    #[inline]
    fn peek_at(&self, pos: PeekFrom) -> Result<Option<&T>, Self::E> where Self: Sized {
        Ok(self.contents().get(calculate_peek_position(self, pos)?))
    }

    fn take(&mut self, target: &T) -> bool {
        match self.contents().get(self.pos()) {
            Some(element) if target == element => {
                self.set_cursor(self.pos() + 1);
                true
            }
            _ => false,
        }
    }

    fn take_multi(&mut self, target: &[&T]) -> bool {
        for &element in target {
            if !self.take(element) { return false }
        }
        true
    }
}

pub trait MutAnalyser<T: Sized + PartialEq + Clone>: Analyser<T> {
    #[inline]
    fn contents_mut(&mut self) -> Result<&mut Vec<T>, Self::E>;

    #[inline]
    fn peek_at_mut(&mut self, pos: PeekFrom) -> Result<Option<&mut T>, Self::E> where Self: Sized {
        let position = calculate_peek_position(self, pos)?;
        Ok(self.contents_mut()?.get_mut(position))
    }

    fn insert_contents(&mut self, contents: &[T]) -> Result<(), Self::E> {
        let pos = self.pos();
        self.contents_mut()?.splice(pos..pos, contents.iter().cloned());
        Ok(())
    }

    fn remove_range(&mut self, range: Range<usize>) -> Result<(), Self::E> {
        return if range.start < range.end && range.end <= self.contents_mut()?.len() {
            self.contents_mut()?.drain(range);
            Ok(())
        } else {
            Err(AnalysisError::InvalidRange(range.start, range.end).into())
        }
    }


}

fn calculate_peek_position<T: Sized + PartialEq + Clone, E: Error + From<AnalysisError>>(analyser: &dyn Analyser<T, E=E>, pos: PeekFrom) -> Result<usize, E>{
    match pos {
        SeekFrom::Start(n) => Ok(n as usize),
        SeekFrom::End(n) => {
            let n = n.abs() as usize;
            if n > analyser.len() {
                Err(AnalysisError::InvalidSeek(n).into())
            } else {
                Ok(analyser.len() - n)
            }
        },
        SeekFrom::Current(n) => {
            let n = n as usize;
            if n > analyser.len() {
                Err(AnalysisError::InvalidSeek(n).into())
            } else {
                Ok(analyser.pos() + n)
            }
        },
    }
}