//! Provides an Iterator<Item=u8> decorator that uses a state machine to exclude comments from a
//! string in linear time and constant space.

pub(crate) trait Exclude<T: Iterator<Item = u8>> {
    fn exclude_comments(self) -> ExcludingComments<T>;
}

impl<T: Iterator<Item = u8>> Exclude<T> for T {
    fn exclude_comments(self) -> ExcludingComments<T> {
        ExcludingComments::new_from_iter(self)
    }
}

pub(crate) struct ExcludingComments<I: Iterator<Item = u8>> {
    state: State,
    buffer: Option<I::Item>,
    iter: I,
}

impl<I: Iterator<Item = u8>> ExcludingComments<I> {
    fn new_from_iter(iter: I) -> Self {
        Self {
            state: State::Char,
            buffer: None,
            iter,
        }
    }
}

enum State {
    Char,
    PotentialLineComment(Option<u8>),
    LineComment,
}

impl<I: Iterator<Item = u8>> Iterator for ExcludingComments<I> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next_option = self.buffer.take().or_else(|| self.iter.next());
            let next = next_option?;

            return match self.state {
                State::Char => match next {
                    b'/' => {
                        self.state = State::PotentialLineComment(next_option);
                        continue;
                    }
                    _ => next_option,
                },
                State::PotentialLineComment(first_slash) => {
                    match next {
                        b'/' => {
                            // second /, enter line comment
                            self.state = State::LineComment;
                            continue;
                        }
                        _ => {
                            // here we need to emit the first /, but save this char for later
                            self.buffer = next_option;
                            self.state = State::Char;
                            return first_slash;
                        }
                    }
                }
                State::LineComment => {
                    match next {
                        b'\n' => {
                            self.state = State::Char;
                            return next_option;
                        }
                        _ => {
                            // ignore all other characters while in the line comment
                            continue;
                        }
                    }
                }
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use std::vec::IntoIter;

    use super::*;

    /// Converts the input to an iterator of u8, excludes comments, maps back to char and collects
    /// the results.
    fn exclude_comments(input: &str) -> String {
        let excluding_comments: ExcludingComments<IntoIter<u8>> = input
            .to_string()
            .into_bytes()
            .into_iter()
            .exclude_comments();
        excluding_comments.map(|b| b as char).collect()
    }

    #[test]
    fn empty() {
        assert!(exclude_comments("").is_empty());
    }

    #[test]
    fn single_char() {
        assert_eq!(exclude_comments("0"), "0");
    }

    #[test]
    fn two_chars() {
        assert_eq!(exclude_comments("ab"), "ab");
    }

    #[test]
    fn comment() {
        assert_eq!(exclude_comments("ab//cd"), "ab");
    }

    #[test]
    fn comments_are_ended_by_new_line() {
        assert_eq!(exclude_comments("ab//comment\nde"), "ab\nde");
    }

    #[test]
    fn new_lines_without_comments() {
        assert_eq!(exclude_comments("ab\nde"), "ab\nde");
    }

    #[test]
    fn single_slash_is_not_excluded() {
        assert_eq!(exclude_comments("ab/cd"), "ab/cd");
    }

    #[test]
    fn multiline() {
        assert_eq!(
            exclude_comments(
                "
line 1 //comment 1
line 2 // comment 2 // comment 3
line 3
line 4 // comment 4"
            ),
            "
line 1 
line 2 
line 3
line 4 "
        );
    }
}
