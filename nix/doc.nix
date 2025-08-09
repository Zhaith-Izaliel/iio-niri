{
  lib,
  runCommandNoCC,
  nixosOptionsDoc,
  iio-niri,
}: let
  # evaluate our options
  eval = lib.evalModules {
    modules = [
      {_module.check = false;}
      (import ./module.nix {inherit iio-niri;})
    ];
  };

  # generate our docs
  optionsDoc = nixosOptionsDoc {
    inherit (eval) options;
  };
in
  # create a derivation for capturing the markdown output
  runCommandNoCC "options-doc.md" {} ''
    cat ${optionsDoc.optionsCommonMark} > $out
  ''
