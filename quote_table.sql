create schema if not exists quote;

create table if not exists quote.quotes (
	id int primary key,
	quote varchar(50),
	author varchar (20)
);
