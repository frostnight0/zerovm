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
        R3 (8bit)  - General Purpose
        R4 (8bit)  - General Purpose
        R5 (8bit)  - General Purpose
        R6 (8bit)  - General Purpose
        R7 (8bit)  - General Purpose
        PC (16bit) - Program Counter
        Z  (bool)  - Zero Flag
        N  (bool)  - Negative Flag
        C  (bool)  - Unsigned Overflow / Carry Flag
        V  (bool)  - Signed Overflow

    Instructions:
        00  NOP

        10  MOV R0, [PC]        // MOV R0, imm8
        11  MOV R1, [PC]        // MOV R1, imm8
        12  MOV R2, [PC]        // MOV R2, imm8
        13  MOV R3, [PC]        // MOV R3, imm8

        20  MOV R0, [R3 R2]     // R0, [(R3 << 8) | R2]
        21  MOV R1, [R3 R2]     // R1, [(R3 << 8) | R2]
        22  MOV R2, [R1 R0]     // R2, [(R1 << 8) | R0]
        23  MOV R3, [R1 R0]     // R3, [(R1 << 8) | R0]

        30  MOV [R3 R2], R0     // [(R3 << 8) | R2], R0
        31  MOV [R3 R2], R1     // [(R3 << 8) | R2], R1
        32  MOV [R1 R0], R2     // [(R1 << 8) | R0], R2
        33  MOV [R1 R0], R3     // [(R1 << 8) | R0], R3

        40  ADD R0, R2          // Set: ZNCV
        41  ADD R1, R3          // Set: ZNCV
        42  ADC R0, R2          // Set: ZNCV
        43  ADC R1, R3          // Set: ZNCV
        44  NOT R0
        45  NOT R1
        46  AND R0, R2
        47  AND R1, R3
        48  XOR R0, R2
        49  XOR R1, R3
        4A  SHR R0
        4B  SHR R1
        4C  SHL R0
        4D  SHL R1
        4E  CMP R0, R2
        4F  CMP R1, R3

        61  MOV R0, R1
        62  MOV R0, R2
        63  MOV R0, R3
        64  MOV R0, R4
        65  MOV R0, R5
        66  MOV R0, R6
        67  MOV R0, R7
        6A  MOV R1, R2
        6B  MOV R1, R3
        6C  MOV R1, R4
        6D  MOV R1, R5
        6E  MOV R1, R6
        6F  MOV R1, R7

        71  MOV R1, R0
        72  MOV R2, R0
        73  MOV R3, R0
        74  MOV R4, R0
        75  MOV R5, R0
        76  MOV R6, R0
        77  MOV R7, R0
        7A  MOV R2, R1
        7B  MOV R3, R1
        7C  MOV R4, R1
        7D  MOV R5, R1
        7E  MOV R6, R1
        7F  MOV R7, R1

        F0  HALT

        Ideas:
          1) PC as pair of GPRS
          2) Rx0 hardwired to zero
          3) Rx for flags
          4) No flags
          5) Stack based CPU

RAM:
    Address: 16bit
    Data:    8bit

GPU:
    text mode: 80x25

Features:
    RAM backend: redis
```
