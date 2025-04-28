use thiserror::Error;

use crate::runtime::types;

#[derive(Error, Debug)]
pub enum RuntimeError<'a> {
    //  Otherwise, if the resolved field is not a static (class) field or an interface field, getstatic throws an IncompatibleClassChangeError.
    //#[error("IncompatibleClassChangeError")]
    //IncompatibleClassChangeError,
    #[error["Current Frame not found"]]
    FrameNotFound,

    #[error["Wrong Type (expected {0}, got {1})"]]
    WrongType(&'a str, types::Type),

    #[error["Wrong Instance (expected {0}, got {1:?})"]]
    WrongInstance(&'a str, types::Instance),

    #[error["NullPointer"]]
    NullPointerException,

    #[error["GeneralException ({0})"]]
    GeneralException(&'a str),

    #[error["CloneNotSupported ({0})"]]
    CloneNotSupported(String),

    //#[error["ArrayStoreException"]]
    //ArrayStoreException,
    #[error["ArrayIndexOutOfBoundsException {0} out of {1}"]]
    ArrayIndexOutOfBoundsException(usize, usize),

    #[error["ArithmeticException"]]
    ArithmeticException,

    #[error["NotImplementException"]]
    NotImplementedException,

    #[error["ClassNotFound {0}"]]
    ClassNotFoundException(String),
}
