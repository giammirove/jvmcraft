import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

public class Demo {
  public static void main(String[] args) {
    ControlGroup controlGroup = new ControlGroup();
    controlGroup.addUnits(new Zealot(), new DarkTemplar(), new Unit() {
      @Override
      public int damage() {
        return 4;
      }
    });

    int groupAttackPower = controlGroup.damage();
    System.out.print("Group attack power is ");
    System.out.println(groupAttackPower);
  }
}

interface Unit {
  int damage();
}

abstract class AbstractUnit implements Unit {
  public AbstractUnit() {
    System.out.println(say());
  }

  protected abstract String say();
}

class Zealot extends AbstractUnit {
  @Override
  public int damage() {
    return 8;
  }

  @Override
  public String say() {
    return "We embrace the glory of battle!";
  }
}

class DarkTemplar extends AbstractUnit {
  @Override
  public int damage() {
    return 45;
  }

  @Override
  public String say() {
    return "Battle is upon us!";
  }
}

class ControlGroup implements Unit {
  private final List<Unit> units = new ArrayList<>();

  public void addUnits(Unit... units) {
    this.units.addAll(Arrays.asList(units));
  }

  @Override
  public int damage() {
    int totalAttackPower = 0;
    for (Unit unit : units) {
      totalAttackPower += unit.damage();
    }
    return totalAttackPower;
  }
}

