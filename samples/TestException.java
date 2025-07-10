public class TestException {

  public static float division(float x, float y) {
    return x / y;
  }

  public static float throwing(float x, float y) {
    throw new ArithmeticException();
  }

  public static float throwingandcatch(float x, float y) {
    throw new ArithmeticException();
  }

  public static void main(String[] args) {
    try {
      division(20, 0);
      // throwing(20, 0);
      // throwingandcatch(20, 0);
      // this will call java/lang/invoke/StringConcatFactory.makeConcatWithConstants
      // System.out.println("yeee " + division(20, 1));
    } catch (Exception e) {
      System.out.println("Something went wrong");
    }
  }
}
