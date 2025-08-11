# NixOS Options Documentation

This file highlights the documentation of every option provided by this flake's module.


## service\.iio-niri\.enable



Whether to enable IIO-Niri\.



*Type:*
boolean



*Default:*
` false `



*Example:*
` true `



## service\.iio-niri\.package



The iio-niri package to use\.



*Type:*
package



*Default:*
` <derivation iio-niri-1.2.0> `



## service\.iio-niri\.extraArgs



Extra arguments to pass to IIO-Niri\.



*Type:*
list of string



*Default:*
` [ ] `



## service\.iio-niri\.niriUnit



The Niri **user** service unit to bind IIO-Niri’s **user** service unit to\.



*Type:*
non-empty string



*Default:*
` "niri.service" `


