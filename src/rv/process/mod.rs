pub mod io; pub use io::*;
pub mod error;pub use error::*;
pub mod define; pub use define::*;
use std::io::{Error, Read, Seek, Write};
use byteorder::WriteBytesExt;
use vfs::FileSystem;

type PreprocessorResult<O> = Result<O, PreprocessError>;
type PreprocessorVoidResult = PreprocessorResult<()>;


struct Preprocessor {
    filesystem: Box<dyn FileSystem>,
    macros:     Vec<Macro>
}

impl Preprocessor {
    pub fn process_path<I: Read + Seek, O: Write>(&mut self,
      output: &mut Option<&mut O>,
      path: &String
    ) -> PreprocessorVoidResult {
        self.follow_include::<I, O>(output, path)
        //If include not found error convert to path not found
    }

    fn global_scan<I: Read + Seek, O: Write>(&mut self,
      reader: &mut PreprocessorReader<I>,
      output: &mut Option<&mut O>,
      starting_token: LexToken
    ) -> PreprocessorVoidResult {
        let mut current_line: u32 = 0;
        let mut current_token = starting_token;
        let mut quoted = false;
        let mut text_buffer: String = String::new();
        loop {
            match current_token {
                LexToken::DQuote => quoted = !quoted,
                LexToken::NewLine | LexToken::NewFile => {
                    if !quoted {
                        Self::handle_line_file(reader, &mut current_token, &mut current_line, output)?;
                    }
                }
                LexToken::Hash => {
                    if quoted {
                        Self::continue_output(&mut text_buffer, &mut current_token, output, reader)?;
                    } else {
                        self.handle_hash(reader, &mut text_buffer, &mut current_token, &mut current_line, output)?;
                    }
                }
                LexToken::DelimitedCommentStart => {
                    self.consume_block_comment::<I, O>(reader, output, &mut current_line)?;
                    current_token = Self::get_next(reader, &mut text_buffer)?;
                }
                LexToken::LineCommentStart => {
                    self.consume_line_comment::<I, O>(reader, &mut current_token, &mut current_line)?;
                    current_token = Self::get_next(reader, &mut text_buffer)?;
                }
                LexToken::Text(_) => {
                    if !quoted {
                        self.try_expand_macro(&text_buffer, reader, output, &mut current_token, &mut current_line)?;
                    } else {
                        Self::continue_output(&mut text_buffer, &mut current_token, output, reader)?;
                    }
                }
                _ => {
                    match output {
                        Some(out) if !quoted => {
                            out.write(text_buffer.as_bytes())?;
                        }
                        _ => {
                            Self::continue_output(&mut text_buffer, &mut current_token, output, reader)?;
                        }
                    }
                }
            }
        }
    }

    fn handle_hash<I: Read + Seek, O: Write>(&mut self,
      reader: &mut PreprocessorReader<I>,
      text_buffer:   &mut String,
      current_token: &mut LexToken,
      current_line: &mut u32,
      output: &mut Option<&mut O>
    ) -> PreprocessorVoidResult {
        *current_token = Preprocessor::get_next(reader, text_buffer)?;
        reader.skip_whitespace()?;
        if let None = output {
            return Err(PreprocessError::InvalidToken {
                line: *current_line,
                token: current_token.clone()
            })
        }

        return match current_token {
            LexToken::Include => self.consume_include_directive(reader, output, text_buffer, current_token, current_line),
            LexToken::Define => self.consume_define_directive::<I, O>(reader, output, text_buffer, current_token, current_line),
            LexToken::IfDef => self.consume_if_block(reader, output, current_token, current_line),
            LexToken::IfNDef => self.consume_if_not_block(reader, output, current_token, current_line),
            LexToken::Undef => self.consume_undefine_directive(reader, output, current_token, current_line),
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
      output: &mut Option<&mut O>,
      path: &String
    ) -> PreprocessorVoidResult {
        self.global_scan(&mut PreprocessorReader::<I>::new(self.locate_stream(path)?), output, LexToken::NewFile)
    }

    fn consume_include_directive<I: Read + Seek, O: Write>(&mut self,
      reader: &mut PreprocessorReader<I>,
      output: &mut Option<&mut O>,
      text_buffer: &mut String,
      current_token: &mut LexToken,
      current_line: &mut u32
    ) -> PreprocessorVoidResult {
        if match current_token {
            LexToken::DQuote => Ok(reader.scan_string(text_buffer, 127, CONST_DOUBLE_QUOTE)?),
            LexToken::LeftAngle => Ok(reader.scan_string(text_buffer, 127, CONST_RIGHT_ANGLE)?),
            _ => Err(PreprocessError::InvalidToken {
                line: current_line.clone(),
                token: current_token.clone()
            })
        }? > 0 {
            self.follow_include::<I, O>(output, &text_buffer)?;
            *current_token = Preprocessor::get_next(reader, text_buffer)?;
            return Ok(());
        }

        Err(PreprocessError::EmptyInclude(*current_line))
    }

    fn consume_define_directive<I: Read + Seek, O: Write>(&mut self,
      reader: &mut PreprocessorReader<I>,
      output: &mut Option<&mut O>,
      text_buffer: &mut String,
      current_token: &mut LexToken,
      current_line: &mut u32
    ) -> PreprocessorVoidResult {
        *current_token = Preprocessor::get_next(reader, text_buffer)?;
        let name = text_buffer.clone();
        let macro_arguments = self.consume_macro_arguments::<I, O>(reader, text_buffer, current_token)?;
        if *current_token != LexToken::RightParenthesis {
            return Err(PreprocessError::InvalidToken { line: *current_line, token: current_token.clone() })
        } else {
            reader.skip_whitespace()?;
            *current_token = Preprocessor::get_next(reader, text_buffer)?;
        }
        if text_buffer.chars().next().unwrap_or(' ') == ' ' {
            reader.skip_whitespace()?;
            *current_token = Preprocessor::get_next(reader, text_buffer)?;
        }
        let macro_value = self.consume_macro_value::<I, O>(reader, output, text_buffer, current_token, current_line)?;

        self.add_macro(Macro::create_simple(name, macro_arguments, macro_value))
    }

    fn add_macro(&mut self, r#macro: Macro) -> PreprocessorVoidResult {
        Ok(self.macros.push(r#macro))
    }

    fn consume_macro_arguments<I: Read + Seek, O: Write>(&self,
      reader: &mut PreprocessorReader<I>,
      text_buffer: &mut String,
      current_token: &mut LexToken,
    ) -> PreprocessorResult<Vec<String>> {
        let mut arguments: Vec<String> = Vec::new();
        *current_token = Preprocessor::get_next(reader, text_buffer)?;
        if *current_token == LexToken::LeftParenthesis {
            reader.skip_whitespace()?;
            *current_token = Preprocessor::get_next(reader, text_buffer)?;
            while let LexToken::Text(parameter) = current_token.clone() {
                arguments.push(parameter.clone());
                reader.skip_whitespace()?;
                *current_token = Preprocessor::get_next(reader, text_buffer)?;
                if *current_token == LexToken::Comma {
                    *current_token = Preprocessor::get_next(reader, text_buffer)?;
                }
            }
        }

        return Ok(arguments);
    }

    fn consume_macro_value<I: Read + Seek, O: Write>(&self,
      reader: &mut PreprocessorReader<I>,
      output: &mut Option<&mut O>,
      text_buffer: &mut String,
      current_token: &mut LexToken,
      current_line: &mut u32
    ) -> PreprocessorResult<String> {
        let mut value = String::new();
        while *current_token != LexToken::NewLine {
            match current_token {
                LexToken::LineCommentStart => self.consume_line_comment::<I, O>(reader, current_token, current_line)?,
                LexToken::DelimitedCommentStart =>  self.consume_block_comment::<I, O>(reader, output, current_line)?,

                _ => {value += text_buffer;}
            }

            *current_token = Preprocessor::get_next(reader, text_buffer)?;
        }
        Ok(value)
    }

    #[inline(always)]
    fn consume_block_comment<I: Read + Seek, O: Write>(&self,
      reader: &mut PreprocessorReader<I>,
      output: &mut Option<&mut O>,
      current_line: &mut u32
    ) -> PreprocessorVoidResult {
        reader.skip_block_comment(current_line, output)
    }

    fn locate_stream<I: Read + Seek>(&self,
      path: &String
    ) -> PreprocessorResult<I> {
        todo!()
    }

    fn consume_undefine_directive<I: Read + Seek, O: Write>(&self,
      reader: &mut PreprocessorReader<I>,
      output: &Option<&mut O>,
      current_token: &mut LexToken,
      current_line: &mut u32
    ) -> PreprocessorVoidResult {
        todo!()
    }

    fn consume_if_block<I: Read + Seek, O: Write>(&self,
      reader: &mut PreprocessorReader<I>,
      output: &Option<&mut O>,
      current_token: &mut LexToken,
      current_line: &mut u32
    ) -> PreprocessorVoidResult {
        todo!()
    }

    fn consume_if_not_block<I: Read + Seek, O: Write>(&self,
      reader: &mut PreprocessorReader<I>,
      output: &Option<&mut O>,
      current_token: &mut LexToken,
      current_line: &mut u32
    ) -> PreprocessorVoidResult {
        todo!()
    }



    fn consume_line_comment<I: Read + Seek, O: Write>(&self,
      reader: &mut PreprocessorReader<I>,
      current_token: &mut LexToken,
      current_line: &mut u32
    ) -> PreprocessorVoidResult {
        todo!()
    }

    fn try_expand_macro<I: Read + Seek, O: Write>(&self,
      macro_name: &String,
      reader: &mut PreprocessorReader<I>,
      output: &Option<&mut O>,
      current_token: &mut LexToken,
      current_line: &mut u32
    ) -> PreprocessorVoidResult {
        todo!()
    }


    /*static*/ fn continue_output<I: Read + Seek, O: Write>(
      text_buffer: &mut String,
      current_token: &mut LexToken,
      output: &mut Option<&mut O>,
      reader: &mut PreprocessorReader<I>,
    ) -> PreprocessorVoidResult {
        if let Some(ref mut out) = output {
            out.write(&*(text_buffer.clone()).into_bytes())?;
        }
        *current_token = Preprocessor::get_next(reader, text_buffer)?;
        Ok(())
    }

    /*static*/ fn handle_line_file<I: Read + Seek, O: Write>(
      reader: &mut PreprocessorReader<I>,
      current_token: &mut LexToken,
      current_line: &mut u32,
      output: &mut Option<&mut O>,
    ) -> PreprocessorVoidResult {
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

    /*static*/ fn get_next<I: Read + Seek>(
      reader: &mut PreprocessorReader<I>,
      token_buffer: &mut String
    ) -> Result<LexToken, Error> {
        reader.next_token(token_buffer, 128)
    }

}

impl<R: Read + Seek> PreprocessorReader<R> {
    fn skip_block_comment<O: Write>(&mut self,
      line_count: &mut u32,
      output: &mut Option<&mut O>
    ) -> PreprocessorVoidResult {
        let mut current = self.get()?;
        let mut last: u8 = 0;
        while last != b'*' || current != b'/' {
            if current == b'\n' {
                *line_count += 1;
                if let Some(ref mut out) = output {
                    out.write_u8(current.clone())?;
                }
            }
            last = current;
            current = self.get()?;
        }
        Ok(())
    }
}