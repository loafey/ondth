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
    optFlag=""
fi

if cargo build -p $proj --target wasm32-unknown-unknown --release ; then
    done="yo"
else
    echo -e "    ${bold}${red}Error:${normal} compiling \"$proj\" failed!" 
    exit 1
fi
rm -f assets
ln -s "qwaks/$proj/assets/" assets
echo -e "    ${bold}${orange}Compiled${normal} \"$proj\" QWAK file to: \"target/wasm32-unknown-unknown/release/$proj.wasm\"" 
mkdir -p assets/qwaks/
cp "target/wasm32-unknown-unknown/release/$proj.wasm" "assets/qwaks/default.wasm"
echo -e "      ${bold}${orange}Copied${normal} \"$proj\" QWAK file to asset directory" 

# mkdir -p assets/qwaks
# for proj in ${folders}; do
#     cargo build -p $proj --target wasm32-unknown-unknown --release
#     echo -e "    ${bold}${orange}Compiled${normal} \"$proj\" QWAK file to: \"target/wasm32-unknown-unknown/release/$proj.wasm\"" 
#     cp "target/wasm32-unknown-unknown/release/$proj.wasm" "assets/qwaks/$proj.wasm"
#     echo -e "      ${bold}${orange}Copied${normal} \"$proj\" QWAK file to asset directory" 
# done

cargo run $optFlag