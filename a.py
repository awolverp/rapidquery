import rapidquery as rq

sql = (
    rq.Insert("users")
    .values(id=1, name="Ali")
    .on_conflict(
        rq.OnConflict("id").do_update("name"),
    )
)

_, params = sql.build("postgresql")

for p in params:
    print(p.value)
