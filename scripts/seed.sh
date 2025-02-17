#!/bin/sh

# Connect to MongoDB and insert sample data
mongosh mongodb://admin:password123@localhost:27017/admin --eval '
  db = db.getSiblingDB("sampledb");
  db.dropDatabase();
  
  // Create users collection and insert data
  db.users.insertMany([
    {
      name: "John Doe",
      email: "john@example.com",
      age: 30,
      created_at: new Date()
    },
    {
      name: "Jane Smith",
      email: "jane@example.com",
      age: 25,
      created_at: new Date()
    },
    {
      name: "Bob Wilson",
      email: "bob@example.com",
      age: 45,
      created_at: new Date()
    }
  ]);

  // Create products collection and insert data
  db.products.insertMany([
    {
      name: "Laptop",
      price: 999.99,
      category: "Electronics",
      in_stock: true
    },
    {
      name: "Smartphone",
      price: 599.99,
      category: "Electronics",
      in_stock: true
    },
    {
      name: "Headphones",
      price: 99.99,
      category: "Accessories",
      in_stock: false
    }
  ]);

  // Print the counts
  print("Users count: " + db.users.countDocuments());
  print("Products count: " + db.products.countDocuments());
'
