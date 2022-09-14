set PROJECT_FOLDER="%1"
set ACCESSIBLE_FOLDERS="%PROJECT_FOLDER%/resources"

deno run --allow-read="%ACCESSIBLE_FOLDERS%,." --allow-write="%ACCESSIBLE_FOLDERS%" index.ts "%PROJECT_FOLDER%" %2 %3
