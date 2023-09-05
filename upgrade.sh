PROJECT_FOLDER=$1
ACCESSIBLE_FOLDERS="$PROJECT_FOLDER/resources,$PROJECT_FOLDER/mods"

deno run --node-modules-dir --unstable --allow-read --allow-write="$ACCESSIBLE_FOLDERS" src/index.ts $PROJECT_FOLDER $2 $3
