public class TestGeneral {

  static class A {
    public A() {
    }
  }

  static class B extends A {
    public B() {
    }
  }

  static class C {
    public C() {
    }
  }

  static class D extends B {
    public D() {
    }
  }

  public static int _instanceof() {
    String local0 = "hello";
    if (local0 instanceof String) {
      return 1;
    }
    return 0;
  }

  public static int _instanceof_inheritance() {
    A local0 = new B();
    C local1 = new C();
    Object local2 = new A();
    A local3 = new D();
    if (local0 instanceof B) {
      return 1;
    }
    if (local2 instanceof C) {
      return 1;
    }
    return 0;
  }
}
