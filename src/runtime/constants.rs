pub(crate) const _CON_NAMES: &[&str] = &[
  "OP_LOAD",
  "OP_STORE",
  "OP_INVOKE",
  "OP_NEW",
  "OP_CONSTANT",
  "CONV_OP_LIMIT",
];

pub(crate) const _CON_VALUES: &[i32] = &[
  0, 1, 2, 3, 4, 15, // these must match the semantics in your JVM
];

pub(crate) const _REF_NONE: i32 = 0; // null value
pub(crate) const REF_GET_FIELD: i32 = 1;
pub(crate) const REF_GET_STATIC: i32 = 2;
pub(crate) const REF_PUT_FIELD: i32 = 3;
pub(crate) const REF_PUT_STATIC: i32 = 4;
pub(crate) const REF_INVOKE_VIRTUAL: i32 = 5;
pub(crate) const REF_INVOKE_STATIC: i32 = 6;
pub(crate) const REF_INVOKE_SPECIAL: i32 = 7;
pub(crate) const REF_NEW_INVOKE_SPECIAL: i32 = 8;
pub(crate) const REF_INVOKE_INTERFACE: i32 = 9;
pub(crate) const _REF_LIMIT: i32 = 10;

// see java/lang/invoke/MethodHandleNatives$Constants
pub(crate) const MN_IS_METHOD: i32 = 0x00010000; // method (not constructor)
pub(crate) const MN_IS_CONSTRUCTOR: i32 = 0x00020000; // constructor
pub(crate) const MN_IS_FIELD: i32 = 0x00040000; // field
pub(crate) const _MN_IS_TYPE: i32 = 0x00080000; // nested type
pub(crate) const _MN_CALLER_SENSITIVE: i32 = 0x00100000; // @CallerSensitive annotation detected
pub(crate) const _MN_TRUSTED_FINAL: i32 = 0x00200000; // trusted final field
pub(crate) const _MN_HIDDEN_MEMBER: i32 = 0x00400000; // members defined in a hidden class or with @Hidden
pub(crate) const MN_REFERENCE_KIND_SHIFT: i32 = 24; // refKind
pub(crate) const MN_REFERENCE_KIND_MASK: i32 = 0x0F000000 >> MN_REFERENCE_KIND_SHIFT;

#[derive(Debug)]
#[allow(dead_code)]
pub enum ConstantInfo {
  // 0
  Type {},
  // 1
  MethodHandle {
    method_name: String,
    descriptor: String,
  },
  // 2
  MemberName {},
  // 3
  String {},
  VarHandle {
    field_name: String,
    descriptor: String,
  },
  CallSite {
    bootstrap_args: Vec<String>,
  },
}

#[derive(Debug)]
pub(crate) struct Constants {}

impl Constants {
  pub(crate) fn is_method(flags: i32) -> bool {
    flags & MN_IS_METHOD != 0
  }

  pub(crate) fn is_field(flags: i32) -> bool {
    flags & MN_IS_FIELD != 0
  }

  pub(crate) fn is_constructor(flags: i32) -> bool {
    flags & MN_IS_CONSTRUCTOR != 0
  }
}
