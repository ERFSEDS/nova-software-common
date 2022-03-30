# novafc-software

Software for the Nova Flight Computer.
This is composed of three modules: 
* `common` holds the common elements between the flight computer, the simulator, and the verifier.
This includes the state machine, data aquisition, and control
* `flight` is the main for the embedded flight computer
* `simulation` contains an implementations of mock flight computer hardware for simulating executions
of config files on a laptop for verification before flight.
* `data-format` is the format used to store telementary between the flight computer and the ground station
* `config-format` holds the format config files are encoded in between the verifier and the state machine

See submodules for further documentation.
