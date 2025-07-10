#[cfg(test)]

mod tests {

  use crate::runtime::opcode::OpCode;

  #[test]

  fn test_from_byte() {
    assert!(OpCode::from_byte(0) == OpCode::NOP);

    assert!(OpCode::from_byte(0) == OpCode::NOP);

    assert!(OpCode::from_byte(1) == OpCode::ACONSTNULL);

    assert!(OpCode::from_byte(2) == OpCode::ICONSTM1);

    assert!(OpCode::from_byte(3) == OpCode::ICONST0);

    assert!(OpCode::from_byte(4) == OpCode::ICONST1);

    assert!(OpCode::from_byte(5) == OpCode::ICONST2);

    assert!(OpCode::from_byte(6) == OpCode::ICONST3);

    assert!(OpCode::from_byte(7) == OpCode::ICONST4);

    assert!(OpCode::from_byte(8) == OpCode::ICONST5);

    assert!(OpCode::from_byte(9) == OpCode::LCONST0);

    assert!(OpCode::from_byte(10) == OpCode::LCONST1);

    assert!(OpCode::from_byte(11) == OpCode::FCONST0);

    assert!(OpCode::from_byte(12) == OpCode::FCONST1);

    assert!(OpCode::from_byte(13) == OpCode::FCONST2);

    assert!(OpCode::from_byte(14) == OpCode::DCONST0);

    assert!(OpCode::from_byte(15) == OpCode::DCONST1);

    assert!(OpCode::from_byte(16) == OpCode::BIPUSH);

    assert!(OpCode::from_byte(17) == OpCode::SIPUSH);

    assert!(OpCode::from_byte(18) == OpCode::LDC);

    assert!(OpCode::from_byte(19) == OpCode::LDCW);

    assert!(OpCode::from_byte(20) == OpCode::LDC2W);

    assert!(OpCode::from_byte(21) == OpCode::ILOAD);

    assert!(OpCode::from_byte(22) == OpCode::LLOAD);

    assert!(OpCode::from_byte(23) == OpCode::FLOAD);

    assert!(OpCode::from_byte(24) == OpCode::DLOAD);

    assert!(OpCode::from_byte(25) == OpCode::ALOAD);

    assert!(OpCode::from_byte(26) == OpCode::ILOAD0);

    assert!(OpCode::from_byte(27) == OpCode::ILOAD1);

    assert!(OpCode::from_byte(28) == OpCode::ILOAD2);

    assert!(OpCode::from_byte(29) == OpCode::ILOAD3);

    assert!(OpCode::from_byte(30) == OpCode::LLOAD0);

    assert!(OpCode::from_byte(31) == OpCode::LLOAD1);

    assert!(OpCode::from_byte(32) == OpCode::LLOAD2);

    assert!(OpCode::from_byte(33) == OpCode::LLOAD3);

    assert!(OpCode::from_byte(34) == OpCode::FLOAD0);

    assert!(OpCode::from_byte(35) == OpCode::FLOAD1);

    assert!(OpCode::from_byte(36) == OpCode::FLOAD2);

    assert!(OpCode::from_byte(37) == OpCode::FLOAD3);

    assert!(OpCode::from_byte(38) == OpCode::DLOAD0);

    assert!(OpCode::from_byte(39) == OpCode::DLOAD1);

    assert!(OpCode::from_byte(40) == OpCode::DLOAD2);

    assert!(OpCode::from_byte(41) == OpCode::DLOAD3);

    assert!(OpCode::from_byte(42) == OpCode::ALOAD0);

    assert!(OpCode::from_byte(43) == OpCode::ALOAD1);

    assert!(OpCode::from_byte(44) == OpCode::ALOAD2);

    assert!(OpCode::from_byte(45) == OpCode::ALOAD3);

    assert!(OpCode::from_byte(46) == OpCode::IALOAD);

    assert!(OpCode::from_byte(47) == OpCode::LALOAD);

    assert!(OpCode::from_byte(50) == OpCode::AALOAD);

    assert!(OpCode::from_byte(51) == OpCode::BALOAD);

    assert!(OpCode::from_byte(52) == OpCode::CALOAD);

    assert!(OpCode::from_byte(53) == OpCode::SALOAD);

    assert!(OpCode::from_byte(54) == OpCode::ISTORE);

    assert!(OpCode::from_byte(55) == OpCode::LSTORE);

    assert!(OpCode::from_byte(56) == OpCode::FSTORE);

    assert!(OpCode::from_byte(57) == OpCode::DSTORE);

    assert!(OpCode::from_byte(58) == OpCode::ASTORE);

    assert!(OpCode::from_byte(59) == OpCode::ISTORE0);

    assert!(OpCode::from_byte(60) == OpCode::ISTORE1);

    assert!(OpCode::from_byte(61) == OpCode::ISTORE2);

    assert!(OpCode::from_byte(62) == OpCode::ISTORE3);

    assert!(OpCode::from_byte(63) == OpCode::LSTORE0);

    assert!(OpCode::from_byte(64) == OpCode::LSTORE1);

    assert!(OpCode::from_byte(65) == OpCode::LSTORE2);

    assert!(OpCode::from_byte(66) == OpCode::LSTORE3);

    assert!(OpCode::from_byte(67) == OpCode::FSTORE0);

    assert!(OpCode::from_byte(68) == OpCode::FSTORE1);

    assert!(OpCode::from_byte(69) == OpCode::FSTORE2);

    assert!(OpCode::from_byte(70) == OpCode::FSTORE3);

    assert!(OpCode::from_byte(71) == OpCode::DSTORE0);

    assert!(OpCode::from_byte(72) == OpCode::DSTORE1);

    assert!(OpCode::from_byte(73) == OpCode::DSTORE2);

    assert!(OpCode::from_byte(74) == OpCode::DSTORE3);

    assert!(OpCode::from_byte(75) == OpCode::ASTORE0);

    assert!(OpCode::from_byte(76) == OpCode::ASTORE1);

    assert!(OpCode::from_byte(77) == OpCode::ASTORE2);

    assert!(OpCode::from_byte(78) == OpCode::ASTORE3);

    assert!(OpCode::from_byte(79) == OpCode::IASTORE);

    assert!(OpCode::from_byte(80) == OpCode::LASTORE);

    assert!(OpCode::from_byte(83) == OpCode::AASTORE);

    assert!(OpCode::from_byte(84) == OpCode::BASTORE);

    assert!(OpCode::from_byte(85) == OpCode::CASTORE);

    assert!(OpCode::from_byte(86) == OpCode::SASTORE);

    assert!(OpCode::from_byte(87) == OpCode::POP);

    assert!(OpCode::from_byte(88) == OpCode::POP2);

    assert!(OpCode::from_byte(89) == OpCode::DUP);

    assert!(OpCode::from_byte(90) == OpCode::DUPX1);

    assert!(OpCode::from_byte(91) == OpCode::DUPX2);

    assert!(OpCode::from_byte(92) == OpCode::DUP2);

    assert!(OpCode::from_byte(93) == OpCode::DUP2X1);

    assert!(OpCode::from_byte(94) == OpCode::DUP2X2);

    assert!(OpCode::from_byte(95) == OpCode::SWAP);

    assert!(OpCode::from_byte(96) == OpCode::IADD);

    assert!(OpCode::from_byte(97) == OpCode::LADD);

    assert!(OpCode::from_byte(98) == OpCode::FADD);

    assert!(OpCode::from_byte(99) == OpCode::DADD);

    assert!(OpCode::from_byte(100) == OpCode::ISUB);

    assert!(OpCode::from_byte(101) == OpCode::LSUB);

    assert!(OpCode::from_byte(102) == OpCode::FSUB);

    assert!(OpCode::from_byte(103) == OpCode::DSUB);

    assert!(OpCode::from_byte(104) == OpCode::IMUL);

    assert!(OpCode::from_byte(105) == OpCode::LMUL);

    assert!(OpCode::from_byte(106) == OpCode::FMUL);

    assert!(OpCode::from_byte(107) == OpCode::DMUL);

    assert!(OpCode::from_byte(108) == OpCode::IDIV);

    assert!(OpCode::from_byte(109) == OpCode::LDIV);

    assert!(OpCode::from_byte(110) == OpCode::FDIV);

    assert!(OpCode::from_byte(111) == OpCode::DDIV);

    assert!(OpCode::from_byte(112) == OpCode::IREM);

    assert!(OpCode::from_byte(113) == OpCode::LREM);

    assert!(OpCode::from_byte(114) == OpCode::FREM);

    assert!(OpCode::from_byte(115) == OpCode::DREM);

    assert!(OpCode::from_byte(116) == OpCode::INEG);

    assert!(OpCode::from_byte(117) == OpCode::LNEG);

    assert!(OpCode::from_byte(118) == OpCode::FNEG);

    assert!(OpCode::from_byte(119) == OpCode::DNEG);

    assert!(OpCode::from_byte(120) == OpCode::ISHL);

    assert!(OpCode::from_byte(121) == OpCode::LSHL);

    assert!(OpCode::from_byte(122) == OpCode::ISHR);

    assert!(OpCode::from_byte(123) == OpCode::LSHR);

    assert!(OpCode::from_byte(124) == OpCode::IUSHR);

    assert!(OpCode::from_byte(125) == OpCode::LUSHR);

    assert!(OpCode::from_byte(126) == OpCode::IAND);

    assert!(OpCode::from_byte(127) == OpCode::LAND);

    assert!(OpCode::from_byte(128) == OpCode::IOR);

    assert!(OpCode::from_byte(129) == OpCode::LOR);

    assert!(OpCode::from_byte(130) == OpCode::IXOR);

    assert!(OpCode::from_byte(131) == OpCode::LXOR);

    assert!(OpCode::from_byte(132) == OpCode::IINC);

    assert!(OpCode::from_byte(133) == OpCode::I2L);

    assert!(OpCode::from_byte(134) == OpCode::I2F);

    assert!(OpCode::from_byte(135) == OpCode::I2D);

    assert!(OpCode::from_byte(136) == OpCode::L2I);

    assert!(OpCode::from_byte(137) == OpCode::L2F);

    assert!(OpCode::from_byte(138) == OpCode::L2D);

    assert!(OpCode::from_byte(139) == OpCode::F2I);

    assert!(OpCode::from_byte(140) == OpCode::F2L);

    assert!(OpCode::from_byte(141) == OpCode::F2D);

    assert!(OpCode::from_byte(142) == OpCode::D2I);

    assert!(OpCode::from_byte(143) == OpCode::D2L);

    assert!(OpCode::from_byte(144) == OpCode::D2F);

    assert!(OpCode::from_byte(145) == OpCode::I2B);

    assert!(OpCode::from_byte(146) == OpCode::I2C);

    assert!(OpCode::from_byte(147) == OpCode::I2S);

    assert!(OpCode::from_byte(148) == OpCode::LCMP);

    assert!(OpCode::from_byte(149) == OpCode::FCMPL);

    assert!(OpCode::from_byte(150) == OpCode::FCMPG);

    assert!(OpCode::from_byte(151) == OpCode::DCMPL);

    assert!(OpCode::from_byte(152) == OpCode::DCMPG);

    assert!(OpCode::from_byte(153) == OpCode::IFEQ);

    assert!(OpCode::from_byte(154) == OpCode::IFNE);

    assert!(OpCode::from_byte(155) == OpCode::IFLT);

    assert!(OpCode::from_byte(156) == OpCode::IFGE);

    assert!(OpCode::from_byte(157) == OpCode::IFGT);

    assert!(OpCode::from_byte(158) == OpCode::IFLE);

    assert!(OpCode::from_byte(159) == OpCode::IFICMPEQ);

    assert!(OpCode::from_byte(160) == OpCode::IFICMPNE);

    assert!(OpCode::from_byte(161) == OpCode::IFICMPLT);

    assert!(OpCode::from_byte(162) == OpCode::IFICMPGE);

    assert!(OpCode::from_byte(163) == OpCode::IFICMPGT);

    assert!(OpCode::from_byte(164) == OpCode::IFICMPLE);

    assert!(OpCode::from_byte(165) == OpCode::IFACMPEQ);

    assert!(OpCode::from_byte(166) == OpCode::IFACMPNE);

    assert!(OpCode::from_byte(167) == OpCode::GOTO);

    assert!(OpCode::from_byte(170) == OpCode::TABLESWITCH);

    assert!(OpCode::from_byte(171) == OpCode::LOOKUPSWITCH);

    assert!(OpCode::from_byte(172) == OpCode::IRETURN);

    assert!(OpCode::from_byte(173) == OpCode::LRETURN);

    assert!(OpCode::from_byte(174) == OpCode::FRETURN);

    assert!(OpCode::from_byte(175) == OpCode::DRETURN);

    assert!(OpCode::from_byte(176) == OpCode::ARETURN);

    assert!(OpCode::from_byte(177) == OpCode::RETURN);

    assert!(OpCode::from_byte(178) == OpCode::GETSTATIC);

    assert!(OpCode::from_byte(179) == OpCode::PUTSTATIC);

    assert!(OpCode::from_byte(180) == OpCode::GETFIELD);

    assert!(OpCode::from_byte(181) == OpCode::PUTFIELD);

    assert!(OpCode::from_byte(182) == OpCode::INVOKEVIRTUAL);

    assert!(OpCode::from_byte(183) == OpCode::INVOKESPECIAL);

    assert!(OpCode::from_byte(184) == OpCode::INVOKESTATIC);

    assert!(OpCode::from_byte(185) == OpCode::INVOKEINTERFACE);

    assert!(OpCode::from_byte(186) == OpCode::INVOKEDYNAMIC);

    assert!(OpCode::from_byte(187) == OpCode::NEW);

    assert!(OpCode::from_byte(188) == OpCode::NEWARRAY);

    assert!(OpCode::from_byte(189) == OpCode::ANEWARRAY);

    assert!(OpCode::from_byte(190) == OpCode::ARRAYLENGTH);

    assert!(OpCode::from_byte(191) == OpCode::ATHROW);

    assert!(OpCode::from_byte(192) == OpCode::CHECKCAST);

    assert!(OpCode::from_byte(193) == OpCode::INSTANCEOF);

    assert!(OpCode::from_byte(194) == OpCode::MONITORENTER);

    assert!(OpCode::from_byte(195) == OpCode::MONITOREXIT);

    assert!(OpCode::from_byte(198) == OpCode::IFNULL);

    assert!(OpCode::from_byte(199) == OpCode::IFNONNULL);
  }
}
