$DATABASE_URL = "postgres://user:password@localhost:5432/my_app_db"
Write-Host "DATABASE_URL path is $DATABASE_URL"
sea-orm-cli generate entity -o server\src\entities --with-serde both -v -u $DATABASE_URL --with-prelude none