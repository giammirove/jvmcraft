
when a get resolve or really any other method
with something like get_method_with_index ...
I always assumed until now that they were not native
BUT this is wrong
native methods are methods, and as such they just be considered

Problem

let's say this is real java code

try {
 invokevirtual blabla // -> this trigger an exception
} catch (e) { // -> this is should catch it
}

but let's say that blabla calls other 5 methods and the last one triggers
the exception, while the 3rd one handles it,


