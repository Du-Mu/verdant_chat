-- Your SQL goes here
CREATE TABLE users (
  uuid CHAR(36) NOT NULL,
  name VARCHAR(255) NOT NULL,
  password VARCHAR(64) NOT NULL,
  permission_id INT NOT NULL,
  PRIMARY KEY (uuid)
);
