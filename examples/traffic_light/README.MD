# Traffic Light

**Purpose** Demonstration of a state machine with own state transition rules  

In this example Traffic Light task is a simple state machine which has own state transition rules in calldata that executes on each task's instructions call. 

Calldata logic: check task's state -> transit to nex.

State transition rule: 0(Green)->1(Yellow)->2(Red)->0(Green)

It is an updated traffic_light example with reduced expensive i/o operations (save on each vm.run()).
We initialize vm once with simple instruction to call calldata of 0th task then in loop we reuse it multiple times (vm.run())
Task is saved to persistent storage only once at the end.

### Usage
```cargo run```
