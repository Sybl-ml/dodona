# Introduction to Machine Learning

This guide will give you a quick outline of the field of machine learning. After reading, you'll know enough about machine learning to get started with Sybl.

### What is Machine Learning?

Machine Learning (ML) is a process where a computer program takes in some existing data, investigates the hidden patterns in the data, and then gives us some predictions on new data. 

These programs are called **models**, and there are a number of different models we can use to perform ML on all kinds of data!

These predictions can be really useful: they can help you grow your business, identify gaps in the market, learn about new customer trends and so much more!

### What do we mean by 'data'?

That's a very good question! On the Sybl platform, we allow you to upload data in the form of a Comma-Separated Values file (`.csv`). These files look a little bit like this:

```python
variety,      weight (g), value (GBP)
Granny Smith, 70,         0.15
Gala,         80,         0.2
Cripps Pink,  90,         0.4
..., ..., ...
```

Here we've got a simple dataset that stores information about apples. If your data is currently inside a spreadsheet, you can export it as a `.csv` file using [this guide](https://support.microsoft.com/en-us/office/save-a-workbook-to-text-format-txt-or-csv-3e9a9d6c-70da-4255-aa28-fcacf1f081e6). 

This data tells us the value of an apple based on its characteristics, like its variety and weight in grams. Because this data is **telling us** something about the values of apples, we call it **training data**.   

Let's take a look at another dataset:

```python
variety,     weight (g), value (GBP)
Gala,        50,
Cripps Pink, 60,
Honeycrisp,  50,
...,         ...,
```

At first glance, this data seems to be similar to our last dataset. But look closer: the examples in this dataset are different, and more importantly, **we don't know the value of the examples in this dataset** (we've represented this by making them blank).

This time the data isn't telling us anything about the value of the apples, so it can't be training data. Instead, this is **test data**, because we can use it to test out our ML!

After running both datasets through an ML program, we might get something like this:

```python
variety,     weight (g), value (GBP)
Gala,        50,         0.15
Cripps Pink, 60,         0.3
Honeycrisp,  50,         0.1
...,         ...,        ...
```

The ML model was able to **learn about the value of apples** from the **training data** and then use this to **predict the value of apples** in the **test data**. Brilliant!

### What if we don't want to predict numbers?

In the previous example, the ML model used our training data to predict the values of specific apples. These values were given as decimal numbers because the attribute (value) was measured in a currency (GBP) that can take decimal values. When we want to use ML to predict numerical values that can be decimals, this is called **regression**. But what about we don't want to predict numbers?

Let's have a look at another training dataset:

```python
colour, taste, variety
Red,    Juicy, Honeycrisp
Green,  Sharp, Granny Smith
Red,    Sweet, Cripps Pink
...,    ...,   ...
```

And a matching test dataset:

```python
colour, taste,    variety
Orange, Aromatic,
Red,    Aromatic,
Orange, Sweet,
...,    ...,      ...
```

In these datasets, the column we want to predict (variety) isn't made up of numbers - and an apple variety certainly can't take decimal values. In fact, none of the attributes of these datasets take numerical values - they are all **categorical** attributes.

Here, we want the ML model to decide which **class** of variety (e.g. Granny Smith, Honeycrisp or Cripps Pink) is most likely or appropriate given the other attributes: colour and taste. This is called **classification**, in contrast to regression.

After performing classification on this dataset, we might get something like this:

```python
colour, taste,    variety
Orange, Aromatic, Cox's Orange Pippin
Red,    Aromatic, Meridian
Orange, Sweet,    Gala
...,    ...,      ...
```

But wait - does this mean that we can't perform classification if the prediction attribute is numerical? The answer is no; sometimes it makes sense for us to perform classification on numerical columns **if the values can't be decimals**. For example:

```python
apples per week, variety,     house number
4,               Gala,        12
7,               Honeycrisp,  19
2,               Cripps Pink, 4
```

If we wanted to predict the house number in the above dataset, we wouldn't want to use regression because asking the ML model to predict a decimal house number wouldn't make sense! Instead, we should use classification to ensure we only predict whole number values.

An alternative way of understanding classification and regression is that **classification can only predict exact values of the column taken from the training data**, whereas **regression can predict any value as long as its a decimal number**.

### What's the catch?

So far, so good! ML is an incredibly powerful tool that can be used to perform ground-breaking academic research, improve our quality of life and add value to our businesses.

But before getting started with ML, it's very important to understand the reasons why **ML is not perfect** and the ways we can try to minimise these risks when working in data science:

- **ML models are only as good as the data it's given**. Imagine we wanted to make another apple variety classifier, but this time, our training data only contains data for one specific apple. The ML model will be able to identify that single apple perfectly, but won't know how to react to other apples that it hasn't seen before. This is because our training data is **too small**; we can try to avoid this by making sure that our training data is as **large** as possible.
- **ML models can inherit prejudices from our data**. Let's stick with the apple variety classifier, but this time, we use training data that is exclusively focused on the Honeycrisp variety (our personal favourite). After training on this data, our model would be great at identifying Honeycrisp apples, but rubbish at identifying any other variety of apple. This is because our training data is **biased**; we can try to avoid this by actively trying to **identify biases** in our training data before we give it to an ML model.
- **ML models are autonomous**. This just means that, by default, they operate with zero human oversight. As great as this might sound on paper, the fact that ML models are autonomous means that it's very difficult for them to know when they're wrong. If our apple variety classifier suddenly starts only predicting Honeycrisp for every apple it sees, it's unlikely to realise that something's gone wrong. To combat this, ML systems need **human oversight** to ensure that we can detect their misbehaviour as soon as they start to get confused.

### Many heads are better than one

Another limitation of ML is that some types of ML models just aren't that good at handling certain types of data. In fact, there is no perfect ML model that works better than every other ML model on every type of data - in data science, we call this the [No Free Lunch theorem](https://en.wikipedia.org/wiki/No_free_lunch_theorem).

One way we can get around this problem is to use **many different models** at the same time using a process called **ensemble learning**. All models are given approximately the same training and test data, and hopefully, they give us slightly different predictions as a result of their individual strengths and weaknesses. After we've got results from each model, we can collate their responses to form overall predictions on the test data. The accuracy of the combined predictions is often better than the average of individual accuracies for each model!  

### Summary

Let's go over the key points from this article:

- Machine Learning (ML) is a process where we teach a computer program to identify patterns in **training data**, then make predictions on **test data**.
- We can use ML to predict numbers (**regression**) or categories (**classification**).
- We can improve the performance of an ML model by making sure that it is given a **large** training dataset, that **biases** in the training data are identified and that it operates with **human oversight**.
- We can train multiple different ML models at a time and collate their predictions to take advantage of their unique strengths and weaknesses through **ensemble learning**.
