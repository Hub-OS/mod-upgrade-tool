set PROJECT_FOLDER="%1"
set ACCESSIBLE_FOLDERS="%PROJECT_FOLDER%/resources"

deno run --allow-read=%ACCESSIBLE_FILES% --allow-write=%ACCESSIBLE_FILES% index.ts $PROJECT_FOLDER %2 %3
