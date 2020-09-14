with import <nixpkgs> {};

let 
  fetchGitHashless = args: stdenv.lib.overrideDerivation
  # Use a dummy hash, to appease fetchgit's assertions
  (fetchgit (args // { sha256 = hashString "sha256" args.url; }))

  # Remove the hash-checking
  (old: {
    outputHash     = null;
    outputHashAlgo = null;
    outputHashMode = null;
    sha256         = null;
  });
in stdenv.mkDerivation {
  name = "px4.json";

  src = fetchGitHashless { 
    url = "https://github.com/PX4/Firmware.git";
    fetchSubmodules = true;
  };
  
  nativeBuildInputs = [
    cmake
    python3
  ];

  buildPhase = ''
    ls
    make parameters_metadata
  '';

  patchPhase = ''
    mkdir .git
    
  '';
  installPhase = ''
    mkdir $out
    mv *.json $out/
    chmod 666 $out/*
  '';
}
