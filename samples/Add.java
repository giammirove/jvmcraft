public class Add {
  static int gotme = 3;
  static float gotme2 = 8.0f;
  static float gotme3 = 5.0f;

  public static int add(int a, int b) {
    return a + b + gotme;
  }

  public static float addf(float a, float b) {
    return a + b;
  }

  static class InnerAdd {
    int value;

    int wow = 20;

    public InnerAdd(int _v) {
      value = _v;
    }

    public void setValue(int value) {
      this.value = value;
    }

    public int getValue() {
      return value + wow;
    }
  }

  public static void main() {
    float a = 3.0f;
    float b = 9.0f;
    double d = 20.0;

    InnerAdd test = new InnerAdd(2);

    // System.out.println("ciao");

    float sum = addf(a + gotme2, b + gotme3);
    if (sum == 25.0) {
      int m = add(0, test.getValue());
    }
  }
}
