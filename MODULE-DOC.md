# NixOS Options Documentation



## services\.iio-niri\.enable



Whether to enable IIO-Niri\.



*Type:*
boolean



*Default:*
` false `



*Example:*
` true `



## services\.iio-niri\.package



The iio-niri package to use\.



*Type:*
package



*Default:*
` <derivation iio-niri-2.0.0> `



## services\.iio-niri\.extraArgs



Extra arguments to pass to IIO-Niri\. Note that it is run with the subcommand ` listen `, so the arguments must be arguments usable on ` listen `\.



*Type:*
list of string



*Default:*
` [ ] `



## services\.iio-niri\.niriUnit



The Niri **user** service unit to bind IIO-Niri’s **user** service unit to\.



*Type:*
non-empty string



*Default:*
` "niri.service" `


