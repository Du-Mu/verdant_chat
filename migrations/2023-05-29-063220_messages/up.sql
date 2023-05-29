-- Your SQL goes here
CREATE TABLE messages (
  uuid CHAR(36) NOT NULL,
  time TIMESTAMP NOT NULL,
  content TEXT NOT NULL,
  sender_id CHAR(36) NOT NULL,
  room_id INT NOT NULL,
  PRIMARY KEY (uuid),
  FOREIGN KEY (sender_id) REFERENCES users(uuid),
  FOREIGN KEY (room_id) REFERENCES rooms(id)
);