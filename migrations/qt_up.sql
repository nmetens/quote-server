alter table quotes rename to quotes_v1;

create table if not exists quote.quotes (
	id int primary key,
	quote varchar(200),
	author varchar (100)
);

create table if not exists tags (
    quote_id int not null;
    tag varchar(200) not null,
    foreign key (id) references quotes(id) 
);