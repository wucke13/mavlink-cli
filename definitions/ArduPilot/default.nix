with import <nixpkgs> {};

stdenv.mkDerivation {
  name = "apm.pdef.json";

  src = builtins.fetchGit { url = "https://github.com/ArduPilot/ardupilot"; };
  
  nativeBuildInputs = [
    python3Packages.lxml
  ];

  buildPhase = ''
    cd Tools/autotest/param_metadata
    for target in ArduPlane ArduCopter Rover ArduSub AntennaTracker
    do
      python ./param_parse.py --vehicle $target --format json
      mv apm.pdef.json $target-apm.pdef.json
    done
    ${jq}/bin/jq -s add *.json > apm.pdef.json
    sed s/"Advanceds"/"Advanced"/g -i apm.pdef.json
  '';

  installPhase = ''
    mkdir $out
    mv *.json $out/
    chmod 666 $out/*
  '';
}
