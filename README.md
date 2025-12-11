# zerovm

### Usage
```
./cpu           - run cpu
./cpu <comm>    - run rust commands, example, ./cpu cargo add <...>
./cpu bash      - run rust container

./start_reds    - must be running before you run cpu
```


### Features
```
====================================
          * vm simulator *   
====================================

version: v0.1

CPU:
    Registers:
        R0 (8bit)  - General Purpose
        R1 (8bit)  - General Purpose
        R2 (8bit)  - General Purpose
        R3 (8bit)  - Flags
        PC (16bit) - Program Counter

    Instructions:
        00  NOP
        10  MOV R0, imm8
        11  MOV R1, imm8
        12  MOV R2, imm8
        13  MOV R3, imm8
        F0  HALT

RAM:
    Backend: - redis
    Address: - 16bit
    Data:    - 8bit

GPU:
    text mode: 80x25
```
