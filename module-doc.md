# NixOS Options Documentation

This file highlights the documentation of every option provided by this flake's module.


## programs\.iio-niri\.enable



Whether to enable IIO-Niri\.



*Type:*
boolean



*Default:*
` false `



*Example:*
` true `



## programs\.iio-niri\.package



The iio-niri package to use\.



*Type:*
package



*Default:*
` <derivation iio-niri-1.1.0> `



## programs\.iio-niri\.service\.enable



Whether to enable the systemd user service to run IIO-Niri\.



*Type:*
boolean



*Default:*
` false `



*Example:*
` true `



## programs\.iio-niri\.service\.extraArgs



Extra arguments to pass to IIO-Niri\.



*Type:*
list of string



*Default:*
` [ ] `



## programs\.iio-niri\.service\.niriUnit



The Niri **user** service unit to bind IIO-Niri’s **user** service unit to\.



*Type:*
non-empty string



*Default:*
` "niri.service" `


