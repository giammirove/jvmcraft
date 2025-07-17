use thiserror::Error;

use crate::runtime::types;

#[derive(Error, Debug)]
pub enum InternalError<'a> {
  #[error["Current Frame not found"]]
  FrameNotFound,

  #[error["Wrong Type (expected {0}, got {1})"]]
  WrongType(&'a str, types::Type),

  #[error["Wrong Instance (expected {0}, got {1:?})"]]
  WrongInstance(&'a str, types::Instance),

  #[error["GeneralError ({0})"]]
  General(String),

  #[error["InternalException {0}"]]
  Exception(String),

  #[error["NotImplement"]]
  NotImplemented,

  #[error["NativeNotImplement {0} {1} {2}"]]
  NativeNotImplemented(String, String, String),

  #[error["WrongClass (expected {0}, got {1})"]]
  WrongClass(String, String),

  #[error["ClassNotFound {0}"]]
  ClassNotFound(String),

  #[error["CodeNotFound ({0} {1} {2})"]]
  CodeNotFound(String, String, String),

  #[error["MethodNotFound ({0} {1} {2})"]]
  MethodNotFound(String, String, String),

  #[error["SegmentationFault ({0})"]]
  SegmentationFault(u64),
}

#[derive(Error, Debug)]
pub enum JavaException {
  #[error["NullPointer"]]
  NullPointer,

  #[error["CloneNotSupported ({0})"]]
  CloneNotSupported(String),

  #[error["ArrayIndexOutOfBoundsException {0} out of {1}"]]
  ArrayIndexOutOfBounds(usize, usize),

  #[error["ArithmeticException"]]
  Arithmetic,

  #[error["IOException ({0})"]]
  IO(String),

  #[error["FileNotFoundException ({0})"]]
  FileNotFound(String),

  #[error["LinkageError"]]
  LinkageError,

  #[error["AssertionError"]]
  AssertionError,

  #[error["IllegalArgumentException ({0})"]]
  IllegalArgumentException(String),
}

impl JavaException {
  pub(crate) fn convert_java_exception_to_classname(exception: &JavaException) -> &str {
    match exception {
      JavaException::NullPointer => "java/lang/NullPointerException",
      JavaException::CloneNotSupported(_) => "java/lang/CloneNotSupportedException",
      JavaException::ArrayIndexOutOfBounds(_, _) => "java/lang/ArrayIndexOutOfBoundsException",
      JavaException::Arithmetic => "java/lang/ArithmeticException",
      JavaException::IO(_) => "java/lang/IOException",
      JavaException::FileNotFound(_) => "java/lang/FileNotFoundException",
      JavaException::LinkageError => "java/lang/LinkageError",
      JavaException::AssertionError => "java/lang/AssertionError",
      JavaException::IllegalArgumentException(_) => "java/lang/IllegalArgumentException",
    }
  }

  pub(crate) fn convert_classname_to_java_exception(classname: &str, msg: String) -> JavaException {
    match classname {
      "java/lang/NullPointerException" => JavaException::NullPointer,
      "java/lang/CloneNotSupportedException" => JavaException::CloneNotSupported(msg),
      "java/lang/ArrayIndexOutOfBoundsException" => {
        JavaException::ArrayIndexOutOfBounds(usize::MAX, usize::MAX)
      }
      "java/lang/ArithmeticException" => JavaException::Arithmetic,
      "java/lang/IOException" => JavaException::IO(msg),
      "java/lang/FileNotFoundException" => JavaException::FileNotFound(msg),
      "java/lang/LinkageError" => JavaException::LinkageError,
      "java/lang/AssertionError" => JavaException::AssertionError,
      "java/lang/IllegalArgumentException" => JavaException::IllegalArgumentException(msg),
      _ => panic!("exception not handled {} -> '{}'", classname, msg),
    }
  }
}
