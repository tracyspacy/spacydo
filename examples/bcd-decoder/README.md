# BINARY CODED DECIMAL DECODER

<img width="393" height="291" alt="Screenshot 2026-03-02 at 18 39 06" src="https://github.com/user-attachments/assets/794edb73-bca8-4534-891c-70d44e99c570" />

<img width="361" height="395" alt="Screenshot 2026-03-02 at 18 41 56" src="https://github.com/user-attachments/assets/4a77b681-bbe5-4e50-a754-23d1a4dc7857" />

----

**This example emulates BCD decoder circuitry from "Code" by Charles Petzold book** [chapter 18](https://codehiddenlanguage.com/Chapter18/)

**Every logic gate in this example is a persistent stateful task running on a spacydo VM. Each gate carries its own executable instructions, each gate can read input/states of gates and update own state.**
____

## What it is?
BCD stands for Binary Coded Decimal, so it is binary representation of decimal numbers from 0 to 9.


| BCD     | DECIMAL      |
| ------------- | ------------- |
| 0000 | 0 |
| 0001 | 1 |
| 0010 | 2 |
| 0011 | 3 |
| 0100 | 4 |
| 0101 | 5 |
| 0110 | 6 |
| 0111 | 7 |
| 1000 | 8 |
| 1001 | 9 |

**Thus BCD decoder's goal is to decode binary input to decimal number (1001 ->9).**

## How to use it?

run: ```cargo run <bcd>```

for example: ```cargo run 1001``` 

## Implementation using spacydo VM

As shown on schematics, BCD decoder has following elements:
a. process input - 4 bits (it may be result of some external process, or manually set) 
b. 4 inverters/NOT gates
c. 9 AND gates

Every element in our implementation is a spacydo native Task and all state transition logic is contained in tasks calldata.

a. **BITS** SWITCH

State of bits represented by simple tasks without any instructions. It emulates simple switch, so we manually change state of each switch representing BIT from 0 to 1.

Task Bytecode Example:

```
PUSH_STRING BIT_0 PUSH_MAX_STATES 2 PUSH_CALLDATA [ ] T_CREATE S_SAVE
```

b. **INVERTERS**

Each inverter has 1 input (state of some bit), one output (own state). Inverter is represented by task containing calldata that checks state of particular bit and simply changes own state to opposite so if input bit is 0 => inverted bit is 1

Task Bytecode:

```
PUSH_STRING %INV_N% PUSH_MAX_STATES 2 PUSH_CALLDATA [ \
PUSH_U32 %ID% PUSH_TASK_FIELD 1 T_GET_FIELD PUSH_STATE 0 EQ \
IF PUSH_STATE 1 PUSH_U32 %OWN_ID% PUSH_TASK_FIELD 1 T_SET_FIELD END_CALL THEN \
PUSH_U32 %ID% PUSH_TASK_FIELD 1 T_GET_FIELD PUSH_STATE 1 EQ \
IF PUSH_STATE 0 PUSH_U32 %OWN_ID% PUSH_TASK_FIELD 1 T_SET_FIELD END_CALL THEN  \
] T_CREATE S_SAVE
```

c. **AND GATES**

Each of AND gates has 4 inputs (state of bits and/or inverted bits) and single output (own state). AND gate is represented by task containing calldata that checks if all 4 inputs has state 1, if so it changes own state to 1. For example, AND gate 1 has following inputs: bit0,inverted_bit_1,inverted_bit_2,inverted_bit_3. 

Task Bytecode:

```
PUSH_STRING %AND_GATE% PUSH_MAX_STATES 2 PUSH_CALLDATA [ \
PUSH_U32 %ID_0% PUSH_TASK_FIELD 1 T_GET_FIELD PUSH_STATE 1 NEQ \
IF \
    PUSH_STATE 0 PUSH_U32 %OWN_ID% PUSH_TASK_FIELD 1 T_SET_FIELD S_SAVE END_CALL THEN \
PUSH_U32 %ID_1% PUSH_TASK_FIELD 1 T_GET_FIELD PUSH_STATE 1 NEQ \
IF \
    PUSH_STATE 0 PUSH_U32 %OWN_ID% PUSH_TASK_FIELD 1 T_SET_FIELD S_SAVE END_CALL THEN \
PUSH_U32 %ID_2% PUSH_TASK_FIELD 1 T_GET_FIELD PUSH_STATE 1 NEQ \
IF \
    PUSH_STATE 0 PUSH_U32 %OWN_ID% PUSH_TASK_FIELD 1 T_SET_FIELD S_SAVE END_CALL THEN \
PUSH_U32 %ID_3% PUSH_TASK_FIELD 1 T_GET_FIELD PUSH_STATE 1 NEQ \
IF \
    PUSH_STATE 0 PUSH_U32 %OWN_ID% PUSH_TASK_FIELD 1 T_SET_FIELD S_SAVE END_CALL THEN \
PUSH_STATE 1 PUSH_U32 %OWN_ID% PUSH_TASK_FIELD 1 T_SET_FIELD S_SAVE END_CALL \
] T_CREATE S_SAVE
```


