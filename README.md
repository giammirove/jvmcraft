# JVMCraft 

Writing JVM in Rust to play Minecraft.  
Ambitious project to learn rust (trying my best).  

As of today 28/04/2025, this project has been developed as a side project 
after work in three weeks ====> not the best code. :)  
This is my first "big" rust project !

> **_NOTE:_** Only tested on openjdk-23.0.2

## Current state of the project

Milestones:
- [x] System.out.Println (~2M instructions x.x)
- [ ] Sockets
- [ ] Minecraft Server
- [ ] Minecraft 

Since the goal of the project is to run minecraft, I will first implement opcodes/internals
needed to run it, and in a second moment implement what's left.

TODO:
- create class loader logic
- separate class based on the class loader
- init threads
- ~~run initPhase1 and initPhase2 and initPhase3~~
- handle CallSite/MethodHandle/LambdaMetaFactories
- sockets
- continue testing

Some vm properties are obtained from env variables such as :
| vm property    | env variable |
| -------------- | ------------ |
| java.home      | JHOME        |
| user.home      | JUHOME       |
| user.dir       | JUDIR        |
| java.io.tmpdir | JTMPDIR      |

## How to run it

Download the java modules:
```bash
curl -L -o openjdk.tar.gz https://download.java.net/java/GA/jdk23.0.2/6da2a6609d6e406f85c491fcb119101b/7/GPL/openjdk-23.0.2_linux-x64_bin.tar.gz
mkdir openjdk
tar -xzf openjdk.tar.gz -C openjdk --strip-components=1
./openjdk/bin/jimage extract --dir java_modules ./openjdk/lib/modules
```

Run it:
```bash
JHOME=/path/to/java_modules cargo run -- -u samples -c Print -m main -d "()V"
```
where `JHOME` is the folder with the java modules just downloaded.

## What it is not implemented/supported 

- JFR
- Multithreading (Virtual Thread/Carrier Threads)
- Virtual Object (every object is materialized)
- Multi OS support (Only linux supported)
- IPv6

## Unit testing

In the future, unit tests MUST be written, but at the moment I do not have time for 
them. :)
