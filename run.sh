bold=$(tput bold)
normal=$(tput sgr0)
orange="\e[94m"
red="\e[91m"

# cd qwaks
# folders=$(ls)
# cd ..

proj=$1
if [ -z "$1" ]; then
    echo -e "    ${bold}${red}Error:${normal} please select a project!" 
    exit 1
fi

optFlag=$2
if [ -z "$2" ]; then
    optFlag="dev"
fi

valid="bug dev production"
if [[ ! " $valid " =~ .*\ $optFlag\ .* ]]; then
    echo -e "    ${bold}${red}Error:${normal} invalid input flag! Valid flags are: ${bold}${orange}bug, dev, production${normal}" 
    exit 1
fi
wasmOut="$2"
wasmOpt="$2"
case $optFlag in
    bug)
        wasmOpt="dev"
        wasmOut="debug"
        ;;
    dev)
        wasmOpt="release"
        wasmOut="release"
        ;;
    production)
        wasmOpt="production"
        ;;
esac

if cargo build -p $proj --target wasm32-unknown-unknown --profile $wasmOpt ; then
    done="yo"
else
    echo -e "    ${bold}${red}Error:${normal} compiling \"$proj\" failed!" 
    exit 1
fi
rm -f assets
ln -s "qwaks/$proj/assets/" assets
echo -e "    ${bold}${orange}Compiled${normal} \"$proj\" QWAK file to: \"target/wasm32-unknown-unknown/${wasmOut}/$proj.wasm\"" 
mkdir -p assets/qwaks/
cp "target/wasm32-unknown-unknown/${wasmOut}/$proj.wasm" "assets/qwaks/default.wasm"
echo -e "      ${bold}${orange}Copied${normal} \"$proj\" QWAK file to asset directory" 


cargo $optFlag --