# launch app

```
  # start database
  cd rundir && surreal start file://db --allow-all --auth --user root --pass root
  # build binary
  cargo watch -x build
  # run (in a different terminal)
  cd rundir && ../target/debug/collabobot
```

# database stuff
```
  # connect as a client to db
  surreal sql -u root -p root --namespace namespace --database database
```

# helpful surrealql phrases
```
  info for db;
  select * from [tablename];
```
