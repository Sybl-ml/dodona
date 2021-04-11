<template>
  <b-container class="p-0 m-0" fluid>
    <b-row class="justify-content-center">
      <b-col xl="12">
        <b-row
          class="highlight_price py-5 mb-3 justify-content-center"
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
              <b-icon class="pricing_icon" font-scale="4" icon="graph-up" />
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
              <b-icon class="pricing_icon" font-scale="4" icon="calculator-fill" />
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
              <b-icon class="pricing_icon" font-scale="4" icon="box-arrow-in-down" />
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
              <b-icon class="pricing_icon" font-scale="4" icon="cash-stack" />
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

.highlight_price {
  height: 15em;
  text-align: center;
  background-color: #fb8c38;
  background-image: url("data:image/svg+xml,%3Csvg width='180' height='180' viewBox='0 0 180 180' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M81.28 88H68.413l19.298 19.298L81.28 88zm2.107 0h13.226L90 107.838 83.387 88zm15.334 0h12.866l-19.298 19.298L98.72 88zm-32.927-2.207L73.586 78h32.827l.5.5 7.294 7.293L115.414 87l-24.707 24.707-.707.707L64.586 87l1.207-1.207zm2.62.207L74 80.414 79.586 86H68.414zm16 0L90 80.414 95.586 86H84.414zm16 0L106 80.414 111.586 86h-11.172zm-8-6h11.173L98 85.586 92.414 80zM82 85.586L87.586 80H76.414L82 85.586zM17.414 0L.707 16.707 0 17.414V0h17.414zM4.28 0L0 12.838V0h4.28zm10.306 0L2.288 12.298 6.388 0h8.198zM180 17.414L162.586 0H180v17.414zM165.414 0l12.298 12.298L173.612 0h-8.198zM180 12.838L175.72 0H180v12.838zM0 163h16.413l.5.5 7.294 7.293L25.414 172l-8 8H0v-17zm0 10h6.613l-2.334 7H0v-7zm14.586 7l7-7H8.72l-2.333 7h8.2zM0 165.414L5.586 171H0v-5.586zM10.414 171L16 165.414 21.586 171H10.414zm-8-6h11.172L8 170.586 2.414 165zM180 163h-16.413l-7.794 7.793-1.207 1.207 8 8H180v-17zm-14.586 17l-7-7h12.865l2.333 7h-8.2zM180 173h-6.613l2.334 7H180v-7zm-21.586-2l5.586-5.586 5.586 5.586h-11.172zM180 165.414L174.414 171H180v-5.586zm-8 5.172l5.586-5.586h-11.172l5.586 5.586zM152.933 25.653l1.414 1.414-33.94 33.942-1.416-1.416 33.943-33.94zm1.414 127.28l-1.414 1.414-33.942-33.94 1.416-1.416 33.94 33.943zm-127.28 1.414l-1.414-1.414 33.94-33.942 1.416 1.416-33.943 33.94zm-1.414-127.28l1.414-1.414 33.942 33.94-1.416 1.416-33.94-33.943zM0 85c2.21 0 4 1.79 4 4s-1.79 4-4 4v-8zm180 0c-2.21 0-4 1.79-4 4s1.79 4 4 4v-8zM94 0c0 2.21-1.79 4-4 4s-4-1.79-4-4h8zm0 180c0-2.21-1.79-4-4-4s-4 1.79-4 4h8z' fill='%23ffffff' fill-opacity='0.46' fill-rule='evenodd'/%3E%3C/svg%3E");
  color: white;
}

.title {
  font-weight: bold;
  font-size: 4rem;
}

.pricing_icon {
  margin: 0.3em 0;
  color: #fb8c38;
}
</style>
