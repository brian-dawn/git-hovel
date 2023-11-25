
# Run migrations


    sqlx database create 
    sqlx migrate run


# Testing locally


NOTE this doesn't work yet, but some day it might!


    git remote add test ssh://git@localhost:2222/git-hovel
    git push test main
