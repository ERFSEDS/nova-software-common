# novafc-software

Software for the Nova Flight Computer.
This is composed of three modules: 
* `common` holds the common elements between the flight computer, the simulator, and the verifier.
This includes the config file format, state machine, and state managment
* `flight` is the main for the embedded flight computer
* `simulation` contains an implementations of mock flight computer hardware for simulating executions
of config files on a laptop for verification before flight.

See submodules for further documentation.
