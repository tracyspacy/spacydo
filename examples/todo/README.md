# todo

**Purpose** Demonstration of programmble tasks  

This example uses spacydo VM and toml parser for tasks instructions.

### Usage

`cargo run -- <arguments>`

basic commands:
- **ls**  - list all tasks
- **show**  - list all tasks & call task's instructions
- **delete** + **task_id** - delete task from persistant storage `cargo run -- delete 1`
- **status** + **task_id** + **new_status** - update task's status `cargo run -- status 1 complete`  Status options: `complete`, `inprogress`, `notcomplete`  
- **add** + **title** + **!optional** **-calldata** + **alias** + **parameters** - create task
example simple task `cargo run -- add Task1`
example task with instructions: `cargo run -- add HidableTask -calldata hide 0` - task will be hide until task with id 0 is not complete

**-calldata** instructions available here [calldata.toml](src/calldata.toml). 

- **chain** - Creates next task once when another completes. Removes calldata after call - called once
example: `cargo run -- add task -calldata chain 5 ChildTask 6` where 5 is task id of task we follow, ChildTask is new task, that would be created when task id 5 will be completed and 6 is task id of this task we creating right now

- **either** Set complete if either one or another task is completed + delete not completed task
example: `cargo run -- add ChooseFruit -calldata either 9 10 11` where 9 is task id for Apple and 10 task id for Orange, 11 is task id for current task we created. If for example Apple status become "complete" -> task 10 (Orange) will be deleted and task 11 ChooseFruit will be set as complete.

- **destructable** - task self destructs as it's status set to complete
example: `cargo run -- add Task -calldata destructable 12` Where 12 is task id of this task 
  
- **hide** - Hide task until another task's status is complete.
example: `cargo run -- add HidableTask -calldata hide 1` Where 1 is task id of task which status we follow

Name aliases for instructions could be changed to any.
New instructions could be add.
![todo_either](https://github.com/user-attachments/assets/6da88ad1-6df3-4445-b529-5824343ea7fb)

 
