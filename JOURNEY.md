# JOURNEY

## Day 1

Trying to run some very simple function to add integer like:
```java

  public static int add(int a, int b) {
    return a + b;
  }

```


## Day 2

Trying to run `System.out.Println`.
But where is `System.out.Println` ??? 

In general arguments of function calls are like
```
(ref to object, [arg1, arg2, ...])
```


## Day 3

Loading the classes should be done by the class loader and not manually ... 
good to know !

So you load `java.lang.Object` and `java.lang.System`.
Then the initialization of the jvm starts.
- init threads (TBD)
- call `initPhase1` in `java.lang.System`
- call `initPhase2` in `java.lang.System`

Then the `Main` can be called using `new ClassLoader("Add")`.
The class loader resolves `Add` in the current directory and loads it.

## Day 4

Field in classes are created and put in the heap.

Each Frame has a stack because not every Frame might clean their stack.
So if using a single stack, the caller would receive garbage stack from the callee.

## Day 5

Checking types when popping from the stack is VERY IMPORTANT.

It is the moment to handle inheritance and subclasses, since StringBuilder is 
a subclass of AbstractStringBuilder and I want to print something.

Big question: how do I want to handle subclasses ?

What I know:
- Method dispatching
```java
Animal a = new Dog();
a.speak();  // JVM uses Dog's vtable entry for speak()
```
- Field Inheritance is resolved statically by type, not by instance
```java
class A { int x = 1; }
class B extends A { int x = 2; }

A obj = new B();
System.out.println(obj.x); // Prints 1
```

Meaning that, an instance of a class has to maintain information about its real class.
Moreover, methods can be overwritten while fields not.
In other words, we can maintain a single vtable per object instance, while
a hierarchical approach should be used for fields.

As of now I will only enforce these typing rules:
```
if B <= A then 
    A := B is ok
    B := A is wrong

A <= Object for all classes A
```

In general, if the a non-static method is called, the first argument is a
reference to the object to call.
```java
dog.speak() // reference to dog is the first argument
```

[!] good to remember: Long and Double take two spaces everywhere (constant pool and stack)

How does a class remember who is she? 
```java
A a = new B(); // how does `a` maintains info about `B` ? 
```

Okey so in `java/lang/Collections` we have `<clinit>`:
```java
  static {};
    descriptor: ()V
    flags: (0x0008) ACC_STATIC
    Code:
      stack=2, locals=0, args_size=0
         0: new           #491                // class java/util/Collections$EmptySet
         3: dup
         4: invokespecial #493                // Method java/util/Collections$EmptySet."<init>":()V
         7: putstatic     #360                // Field EMPTY_SET:Ljava/util/Set;
        10: new           #494                // class java/util/Collections$EmptyList
        13: dup
        14: invokespecial #496                // Method java/util/Collections$EmptyList."<init>":()V
        17: putstatic     #368                // Field EMPTY_LIST:Ljava/util/List;
        20: new           #497                // class java/util/Collections$EmptyMap
        23: dup
        24: invokespecial #499                // Method java/util/Collections$EmptyMap."<init>":()V
        27: putstatic     #372                // Field EMPTY_MAP:Ljava/util/Map;
        30: return
      LineNumberTable:
        line 4624: 0
        line 4750: 10
        line 4854: 20
```

A long as an argument counts as two arguments.

I will not implement any Java Flight Recorder (JFR) function.

## Day 6

Damn if I like writing VM and working with languages!

Stuck or dont want to read opcode using `javap` ? 
Just read the actual code at `https://github.com/openjdk/jdk`

## Day 7

Time to write some unit tests -- debuggin is painful :)

## Day 8

Okey so Unit tests helped to find some missing parts in the integer logic.
But apparently the bug I was looking at is caused by JDK-23 trying to 
access a field that might be null !
java.class.version
To be specific in `https://github.com/openjdk/jdk/blob/jdk-23%2B37/src/java.base/share/classes/jdk/internal/misc/VM.java`
```java
    s = props.get("java.class.version"); // no check if NULL !!!
    int index = s.indexOf('.');
    try {
        classFileMajorVersion = Integer.parseInt(s.substring(0, index));
        classFileMinorVersion = Integer.parseInt(s.substring(index + 1));
    } catch (NumberFormatException e) {
        throw new InternalError(e);
    }
```
And suspiciously, those few lines are not present in JDK-25 ! :p

## Day 9

HashMap are the worst, also the jdk code is something redundant with minor modifications
Example
```java
// from java/util/concurrent/ConcurrentHashMap.java
    static final int HASH_BITS = 0x7fffffff; // usable bits of normal node hash

    static final int spread(int h) {
        return (h ^ (h >>> 16)) & HASH_BITS;
    }

    public V get(Object key) {
        Node<K,V>[] tab; Node<K,V> e, p; int n, eh; K ek;
        int h = spread(key.hashCode());
        if ((tab = table) != null && (n = tab.length) > 0 &&
            (e = tabAt(tab, (n - 1) & h)) != null) {
            if ((eh = e.hash) == h) {
                if ((ek = e.key) == key || (ek != null && key.equals(ek)))
                    return e.val;
            }
            else if (eh < 0)
                return (p = e.find(h, key)) != null ? p.val : null;
            while ((e = e.next) != null) {
                if (e.hash == h &&
                    ((ek = e.key) == key || (ek != null && key.equals(ek))))
                    return e.val;
            }
        }
        return null;
    }

```
```java
// from java/util/HashMap.java
    static final int hash(Object key) {
        int h;
        return (key == null) ? 0 : (h = key.hashCode()) ^ (h >>> 16);
    }
    public V get(Object key) {
        Node<K,V> e;
        return (e = getNode(key)) == null ? null : e.value;
    }

    /**
     * Implements Map.get and related methods.
     *
     * @param key the key
     * @return the node, or null if none
     */
    final Node<K,V> getNode(Object key) {
        Node<K,V>[] tab; Node<K,V> first, e; int n, hash; K k;
        if ((tab = table) != null && (n = tab.length) > 0 &&
            (first = tab[(n - 1) & (hash = hash(key))]) != null) {
            if (first.hash == hash && // always check first node
                ((k = first.key) == key || (key != null && key.equals(k))))
                return first;
            if ((e = first.next) != null) {
                if (first instanceof TreeNode)
                    return ((TreeNode<K,V>)first).getTreeNode(hash, key);
                do {
                    if (e.hash == hash &&
                        ((k = e.key) == key || (key != null && key.equals(k))))
                        return e;
                } while ((e = e.next) != null);
            }
        }
        return null;
    }
```
In particular look at `hash` and `spread(key.hashCode)`.
They return the same value but it is not intuitive.
Moreover, HashMap could be ConcurrentHashMap, therefore why `get` is different?

## Day 10

What does it mean if a field is not in the fields section of a class file, but there's a field reference in the constant pool?
POV: the jvm is a pile of legay. You want `java/lang/Thread.getNextThreadIdOffset()` but
not `nextThreadId` field.
 
TODO resolve my confusion about array and types

[B is not [java/lang/Byte !!!

Suprise suprise some field are manually injected by the JVM, for instance:
- jdk/internal/misc/UnsafeConstants.java - ADDRESS_SIZE0
- jdk/internal/misc/UnsafeConstants.java - PAGE_SIZE

Off-heap -> allocated by native (no GC => manual free)

## Day 11

Dont trust java, if it has to mmap something with no permission => it will read from it anyway !

## Day 12

The number of the day is not really indicative. I took a break from this project.

Fun fact, some code appears as redundant in the JDK as :
```java

    public <U> Class<? extends U> asSubclass(Class<U> clazz) {
        if (clazz.isAssignableFrom(this)) // checked here
            return (Class<? extends U>) this;
        else
            throw new ClassCastException(this.toString());
    }

    @CallerSensitiveAdapter
    private static boolean registerAsParallelCapable(Class<?> caller) {
        if ((caller == null) || !ClassLoader.class.isAssignableFrom(caller)) { // and here
            throw new IllegalCallerException(caller + " not a subclass of ClassLoader");
        }
        return ParallelLoaders.register(caller.asSubclass(ClassLoader.class));
    }
```
In this code, `isAssignableFrom` is checked twice with same parameters!

The JVM sets up tons of internals !!!! :)

Imagine beign the JVM and using classes even for primitive types but not really and 
they have two names (`I` and `int`, `F` and `float`, etc).

## Day 13

Yeeeeeeee `System.out.Println` worksss! 
Next step Socketsss
