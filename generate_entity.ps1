if (Test-Path src/entity/) {
    Remove-Item entity/src/* -r -force
}
sea generate entity -o entity/src -l