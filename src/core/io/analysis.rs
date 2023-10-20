use std::io;
use std::io::{SeekFrom};
use std::ops::{Index, IndexMut, Range};

pub type PeekFrom = SeekFrom;

impl<T: Sized + PartialEq + Clone> Index<Range<usize>> for dyn Analyser<T> {
    type Output = [T];

    fn index(&self, range: Range<usize>) -> &Self::Output {
        &self.contents()[range]
    }
}

impl<T: Sized + PartialEq + Clone> Index<Range<usize>> for dyn MutAnalyser<T> {
    type Output = [T];

    fn index(&self, range: Range<usize>) -> &Self::Output {
        &self.contents()[range]
    }
}

impl<T: Sized + PartialEq + Clone> IndexMut<Range<usize>> for dyn MutAnalyser<T> {
    fn index_mut(&mut self, range: Range<usize>) -> &mut Self::Output {
        &mut self.contents_mut().unwrap()[range]
    }
}

pub trait Analyser<T: Sized + PartialEq + Clone> {
    #[inline]
    fn contents(&self) -> &Vec<T>;

    #[inline]
    fn pos(&self) -> usize;

    #[inline]
    fn set_cursor(&mut self,
      cursor: usize
    );

    #[inline]
    fn len(&self) -> usize { self.contents().len() }

    #[inline]
    fn peek(&self) -> Option<T> { self.contents().get(self.pos()).cloned() }

    #[inline]
    fn is_end(&self) -> bool { self.pos() >= self.len() }

    #[inline]
    fn reset(&mut self) -> usize {
        let old = self.pos();
        self.set_cursor(0);
        return old
    }

    fn seek(&mut self,
      pos: SeekFrom
    ) -> io::Result<usize> where Self: Sized  {
        let position = calculate_peek_position(self, pos)?;

        if position < 0 { return Err(io::Error::other("Invalid Seek")); }
        self.set_cursor(position);
        Ok(self.pos())
    }

    fn get_range(&mut self, from: usize, to: usize) -> &[T] {
        &self.contents()[from..to]
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

    fn get_to(&mut self,
      pos: SeekFrom
    ) -> io::Result<Option<T>> where Self: Sized {
        let position = calculate_peek_position(self, pos)? - 1;

        if position < 0 { return Err(io::Error::other("Invalid Seek")); }

        self.set_cursor(position);
        Ok(self.get())
    }

    #[inline]
    fn peek_at(&self,
      pos: PeekFrom
    ) -> io::Result<Option<&T>> where Self: Sized {
        Ok(self.contents().get(calculate_peek_position(self, pos)?))
    }

    fn take(&mut self, target: &T) -> bool {
        match self.contents().get(self.pos()) {
            Some(element) if *element == *target => {
                self.set_cursor(self.pos() + 1);
                true
            }
            _ => false,
        }
    }

    fn take_multi(&mut self,
      target: &[&T]
    ) -> bool {
        for &element in target {
            if !self.take(element) { return false }
        }
        true
    }
    fn step_back(&mut self) -> Option<T> {
        self.set_cursor(self.pos() - 1);
        self.peek()
    }
}

pub trait MutAnalyser<T: Sized + PartialEq + Clone>: Analyser<T> {
    #[inline]
    fn contents_mut(&mut self) -> io::Result<&mut Vec<T>>;

    #[inline]
    fn peek_at_mut(&mut self,
      pos: PeekFrom
    ) -> io::Result<Option<&mut T>> where Self: Sized {
        let position = calculate_peek_position(self, pos)?;
        Ok(self.contents_mut()?.get_mut(position))
    }

    fn insert_contents(&mut self,
      contents: &[T]
    ) -> io::Result<()> {
        let pos = self.pos();
        self.contents_mut()?.splice(pos..pos, contents.iter().cloned());
        Ok(())
    }

    fn remove_range(&mut self,
       range: Range<usize>
    ) -> io::Result<()> {
        return if range.start < range.end && range.end <= self.contents_mut()?.len() {
            self.contents_mut()?.drain(range);
            Ok(())
        } else {
            Err(io::Error::other("Invalid range"))
        }
    }


}

fn calculate_peek_position<
    T: Sized + PartialEq + Clone
>(
  analyser: &dyn Analyser<T>,
  pos: PeekFrom
) -> io::Result<usize> {
    match pos {
        SeekFrom::Start(n) => Ok(n as usize),
        SeekFrom::End(n) => {
            let n = n.abs() as usize;
            if n > analyser.len() {
                Err(io::Error::other("Invalid Seek"))
            } else {
                Ok(analyser.len() - n)
            }
        },
        SeekFrom::Current(n) => {
            let n = n as usize;
            if n > analyser.len() {
                Err(io::Error::other("Invalid Seek"))
            } else {
                Ok(analyser.pos() + n)
            }
        },
    }
}