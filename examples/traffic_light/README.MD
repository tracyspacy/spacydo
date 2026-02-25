# Traffic Light

**Purpose** Demonstration of a state machine with own state transition rules  

In this example Traffic Light task is a simple state machine which has own state transition rules in calldata that executes on each task's instructions call. 
Calldata logic: check task's state -> transit to next -> save. 
State transition rule: 0(Green)->1(Yellow)->2(Red)->0(Green)

### Usage
```cargo run```
