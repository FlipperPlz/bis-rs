pub mod io; pub use io::*;

use std::io::{Error, Read, Seek, Write};
use async_std::io::WriteExt;
use byteorder::WriteBytesExt;
use thiserror::Error;
use vfs::FileSystem;

type VoidResult = Result<(), PreprocessError>;

struct Preprocessor {
    filesystem: Box<dyn FileSystem>,
}

#[derive(Error, Debug)]
pub enum PreprocessError {
    #[error(transparent)]
    IO(#[from] Error),
    #[error("[{line}] Invalid preprocessor directive \"{directive_text}\"")]
    InvalidDirective {
        line:           u32,
        directive_text: String,
    },
    #[error("[{line}] Invalid preprocessor token \"{token:?}\"")]
    InvalidToken {
        line:           u32,
        token: LexToken,
    },
    #[error("[{0}] Include references an empty path.")]
    EmptyInclude(u32),
    #[error("[{0}] Found endif directive outside of if block")]
    WierdEndif(u32),
    #[error("[{0}] Found else directive outside of if block")]
    WierdElse(u32),
}
macro_rules! invalid_directive {
    ($current_line:ident, $directive_text:ident) => {
        Err(PreprocessError::InvalidDirective { 
            line: $current_line,
            directive_text: $directive_text
        });
    };
}


impl Preprocessor {
    pub fn process_path<I: Read + Seek, O: Write>(&mut self,
      output: Option<&mut O>,
      path: &String
    ) -> VoidResult {
        self.follow_include::<I, O>(output, path)
        //If include not found error convert to path not found
    }

    fn get_next<I: Read + Seek>(
        mut reader: &mut PreprocessorReader<I>,
        mut token_buffer: &mut String
    ) -> Result<LexToken, Error> {
        reader.next_token(token_buffer, 128)
    }



    fn global_scan<I: Read + Seek, O: Write>(&mut self,
                                             reader: &mut PreprocessorReader<I>,
                                             mut output: Option<&mut O>,
                                             starting_token: LexToken
    ) -> VoidResult {
        let mut current_line: u32 = 0;
        let mut current_token = starting_token;
        let mut quoted = false;
        let mut text_buffer: String = String::new();
        loop {
            match current_token {
                LexToken::DQuote => quoted = !quoted,
                LexToken::NewLine | LexToken::NewFile => {
                    if !quoted {
                        Self::handle_line_file(reader, &mut current_token, &mut current_line, &mut output)?;
                    }
                }
                LexToken::Hash => {
                    if quoted {
                        Self::continue_output(&text_buffer, &mut current_token, &mut output, reader)?;
                    } else {
                        self.handle_hash(reader, &mut current_token, &mut current_line, &mut output)?;
                    }
                }
                LexToken::DelimitedCommentStart => {
                    self.consume_block_comment(reader, &mut output, &mut current_token, &mut current_line)?;
                    current_token = Self::get_next(reader, &mut text_buffer)?;
                }
                LexToken::LineCommentStart => {
                    self.consume_line_comment(reader, &mut output, &mut current_token, &mut current_line)?;
                    current_token = Self::get_next(reader, &mut text_buffer)?;
                }
                LexToken::Text(_) => {
                    if !quoted {
                        self.try_expand_macro(&text_buffer, reader, &mut output, &mut current_token, &mut current_line)?;
                    } else {
                        Self::continue_output(&text_buffer, &mut current_token, &mut output, reader)?;
                    }
                }
                _ => {
                    if quoted {
                        match output {
                            None => {}
                            Some(ref mut out) => {
                                out.write(text_buffer.as_bytes())?;
                            },
                        }
                    } else {
                        Self::continue_output(&text_buffer, &mut current_token, &mut output, reader)?;
                    }
                }
            }
        }
    }

    fn continue_output<I: Read + Seek, O: Write>(
        text_buffer: &String,
        current_token: &mut LexToken,
        output: &mut Option<&mut O>,
        reader: &mut PreprocessorReader<I>,
    ) -> VoidResult {
        if let Some(out) = output {
            out.write(&*(text_buffer.clone()).into_bytes())?;
        }
        *current_token = Preprocessor::get_next(reader, &mut text_buffer.clone())?;
        Ok(())
    }

    fn handle_line_file<I: Read + Seek, O: Write>(
        reader: &mut PreprocessorReader<I>,
        current_token: &mut LexToken,
        current_line: &mut u32,
        output: &mut Option<&mut O>,
    ) -> VoidResult {
        let (extra_lines, increment) = match current_token {
            LexToken::NewLine => {
                let extras = reader.directive_newline_count();
                (extras + 1, extras)
            },
            _ => {
                let extras = reader.directive_newline_count();
                (extras, extras)
            }
        };
        *current_line += extra_lines;
        for _ in 0..increment {
            output.as_deref_mut().map_or(Ok(()), |out| out.write_u8(b'\n'))?
        }
        reader.reset_newline_count();
        reader.skip_whitespace()?;
        Ok(())
    }

    fn handle_hash<I: Read + Seek, O: Write>(&mut self,
      reader: &mut PreprocessorReader<I>,
      current_token: &mut LexToken,
      current_line: &mut u32,
      mut output: &mut Option<&mut O>
    ) -> VoidResult {
        *current_token = Preprocessor::get_next(reader, &mut String::new())?;
        reader.skip_whitespace()?;
        if let None = output {
            return Err(PreprocessError::InvalidToken {
                line: *current_line,
                token: current_token.clone()
            });
        }
        match current_token {
            LexToken::Include => self.consume_include_directive(reader, output.as_deref_mut(), current_token, current_line),
            LexToken::Define => self.consume_define_directive(reader, &mut output, current_token, current_line),
            LexToken::IfDef => self.consume_if_block(reader, &mut output, current_token, current_line),
            LexToken::IfNDef => self.consume_if_not_block(reader, &mut output, current_token, current_line),
            LexToken::Undef => self.consume_undefine_directive(reader, &mut output, current_token, current_line),
            LexToken::Else => Err(PreprocessError::WierdElse(*current_line)),
            LexToken::EndIf => Err(PreprocessError::WierdEndif(*current_line)),
            LexToken::Unknown(s) => Err(PreprocessError::InvalidDirective {
                line: *current_line,
                directive_text: s.to_string()
            }),
            _ => Err(PreprocessError::InvalidToken {
                line: *current_line,
                token: current_token.clone()
            }),
        }
    }

    #[inline(always)]
    fn follow_include<I: Read + Seek, O: Write>(&mut self,
     output: Option<&mut O>,
     path: &String
    ) -> VoidResult {
        self.global_scan(&mut PreprocessorReader::<I>::new(self.locate_stream(path)?), output, LexToken::NewFile)
    }

    fn locate_stream<I: Read + Seek>(&self,
      path: &String
    ) -> Result<I, PreprocessError> {
        todo!()
    }


    fn consume_include_directive<I: Read + Seek, O: Write>(&mut self,
      mut reader: &mut PreprocessorReader<I>,
      output: Option<&mut O>,
      mut current_token: &mut LexToken,
      mut current_line: &mut u32
    ) -> VoidResult {
        if let Some(mut path) = match current_token {
            LexToken::DQuote => reader.scan_string(127, CONST_DOUBLE_QUOTE),
            LexToken::LeftAngle => reader.scan_string(127, CONST_RIGHT_ANGLE),
            _ => return Err(PreprocessError::InvalidToken {
                line: current_line.clone(),
                token: current_token.clone()
            })
        }? {
            self.follow_include::<I, O>(output, &path)?;
            *current_token = Preprocessor::get_next(reader, &mut path)?;

        }
        return Err(PreprocessError::InvalidToken {
            line: current_line.clone(),
            token: current_token.clone()
        })
    }

    fn consume_define_directive<I: Read + Seek, O: Write>(&self,
      mut reader: &mut PreprocessorReader<I>,
      output: &mut Option<&mut O>,
      mut current_token: &mut LexToken,
      mut current_line: &mut u32
    ) -> VoidResult {
        todo!()
    }

    fn consume_undefine_directive<I: Read + Seek, O: Write>(&self,
      mut reader: &mut PreprocessorReader<I>,
      output: &mut Option<&mut O>,
      mut current_token: &mut LexToken,
      mut current_line: &mut u32
    ) -> VoidResult {
        todo!()
    }

    fn consume_if_block<I: Read + Seek, O: Write>(&self,
      mut reader: &mut PreprocessorReader<I>,
      output: &mut Option<&mut O>,
      mut current_token: &mut LexToken,
      mut current_line: &mut u32
    ) -> VoidResult {
        todo!()
    }

    fn consume_if_not_block<I: Read + Seek, O: Write>(&self,
      mut reader: &mut PreprocessorReader<I>,
      output: &mut Option<&mut O>,
      mut current_token: &mut LexToken,
      mut current_line: &mut u32
    ) -> VoidResult {
        todo!()
    }

    fn consume_block_comment<I: Read + Seek, O: Write>(&self,
      mut reader: &mut PreprocessorReader<I>,
      output: &mut Option<&mut O>,
      mut current_token: &mut LexToken,
      mut current_line: &mut u32
    ) -> VoidResult {
        todo!()
    }

    fn consume_line_comment<I: Read + Seek, O: Write>(&self,
      mut reader: &mut PreprocessorReader<I>,
      output: &mut Option<&mut O>,
      mut current_token: &mut LexToken,
      mut current_line: &mut u32
    ) -> VoidResult {
        todo!()
    }

    fn try_expand_macro<I: Read + Seek, O: Write>(&self,
      macro_name: &String,
      mut reader: &mut PreprocessorReader<I>,
      output: &mut Option<&mut O>,
      mut current_token: &mut LexToken,
      mut current_line: &mut u32
    ) -> VoidResult {
        todo!()
    }
}

