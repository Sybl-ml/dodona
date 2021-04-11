<template>
  <b-container class="p-0" fluid>
    <b-row class="justify-content-center">
      <b-col xl="12">
        <b-row
          class="highlight py-5 mb-3 justify-content-center"
          align-v="center"
        >
          <div>
            <h1 class="title"><strong>Pricing</strong></h1>
            <h4>
              We've worked hard to build a business model that works for all of
              our users.
            </h4>
          </div>
        </b-row>
      </b-col>
    </b-row>
    <b-row class="justify-content-center">
      <b-col xl="12">
        <b-row class="justify-content-center mr-0 ml-0">
          <b-col cols="auto" class="m-3">
            <b-card
              tag="article"
              class="mb-2 detail_card bg-light border-0 background"
            >
              <b-icon font-scale="4" icon="graph-up" />
              <h3>No Subscriptions, No Nonsense</h3>

              <b-card-text>
                Sybl lets you buy credits that can be used to run jobs on our
                platform. This means you pay per job, rather than per hour or
                through a subscription model. We like to keep things simple.
              </b-card-text>
            </b-card>
          </b-col>
          <b-col cols="auto" class="m-3">
            <b-card
              tag="article"
              class="mb-2 detail_card bg-light border-0 background"
            >
              <b-icon font-scale="4" icon="calculator-fill" />
              <h3>Only Pay for What You Use</h3>

              <b-card-text>
                Job calculations are based on your dataset size, dimensions and
                the number of models you request from our system. We don't
                charge any additional commissions.
              </b-card-text>
            </b-card>
          </b-col>
        </b-row>
        <b-row class="justify-content-center">
          <b-col cols="auto" class="m-3">
            <b-card
              tag="article"
              class="mb-2 detail_card bg-light border-0 background"
            >
              <b-icon font-scale="4" icon="box-arrow-in-down" />
              <h3>Deposit Anytime</h3>

              <b-card-text>
                You can withdraw your Sybl credits at any time through your
                account settings.
              </b-card-text>
            </b-card>
          </b-col>
          <b-col cols="auto" class="m-3">
            <b-card
              tag="article"
              class="mb-2 detail_card bg-light border-0 background"
            >
              <b-icon font-scale="4" icon="cash-stack" />
              <h3>Make Money with Sybl</h3>

              <b-card-text>
                Are you an expert in data science? Register as a client today
                and get paid for hosting your models on our platform! Find out
                more in our client guides.
              </b-card-text>
            </b-card>
          </b-col>
        </b-row>
      </b-col>
    </b-row>
    <br />
  </b-container>
</template>

<style>
.detail_card {
  max-width: 30rem;
  text-align: center;
}

.highlight {
  height: 15em;
  text-align: center;
  background-color: #fb8c38;
  color: white;
}

.title {
  font-weight: bold;
  font-size: 4rem;
}

.b-icon {
  margin: 0.3em 0;
  color: #fb8c38;
}
</style>
