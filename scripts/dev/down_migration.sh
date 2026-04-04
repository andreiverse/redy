#!/bin/bash

echo DATABASE_URL path is $DATABASE_URL

sea-orm-cli migrate down