# RapidQuery

**_RapidQuery: High-Performance SQL Query Builder for Python_**

RapidQuery is a powerful SQL query builder library designed for Python, combining the simplicity of Python with the raw speed of **Rust**. Build complex SQL queries effortlessly and efficiently, with a library that prioritizes both performance and ease of use.

> [!WARNING]\
> This branch is under development and incomplete.

## TODO

ASTs until being merged into default branch.

### DDL

- **Schemas**
  - [x] CreateSchema
  - [x] DropSchema

- **Tables**
  - [x] CreateIndex
  - [x] DropIndex
  - [x] Column
  - [ ] CreateTable
  - [ ] AlterTable
  - [ ] DropTable

- **Views**
  - [ ] CreateView
  - [ ] DropView

- **Custom types**
  - [ ] CreateType
  - [ ] AlterType
  - [ ] DropType

- **Sequences**
  - [ ] CreateSequence
  - [ ] AlterSequence
  - [ ] DropSequence

- **Functions**
  - [ ] CreateFunction
  - [ ] DropFunction

### DCL

- [ ] Grant
- [ ] Revoke

### DML

- [ ] Insert
- [ ] Update
- [ ] Delete
- [ ] Merge
- [ ] TruncateTable
- [ ] Select

### TCL

- **Transactions**
  - [ ] Transaction

- **Savepoints**
  - [ ] Savepoint

## Development

Contributor setup and review rules are in [CONTRIBUTING.md](./CONTRIBUTING.md).
