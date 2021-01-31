<template>
  <div>
    <b-container fluid>
      <b-row>
        <b-col class="justify-content-center main" md="12" lg="6">
          <h1 class="title">
            Empower your data with intuative Machine Learning
          </h1>
          <h2 class="subtitle">
            Run complex models without any infrastructure or programming
            experience. Works with
            <a href="https://www.tensorflow.org/"><tensor-flow-icon /> </a>
            &
            <a href="https://keras.io/"><keras-icon /></a>
          </h2>
          <b-button class="fix-button" variant="primary" to="/register">
            GET STARTED
          </b-button>
          <b-button class="outline-fix-button" variant="primary">
            EXAMPLE
          </b-button>
        </b-col>
        <b-col lg="6" sm="0" class="d-none d-lg-block main justify-content-center">
          <placeholder />
        </b-col>
      </b-row>
    </b-container>
    <div class="highlight">
      <div class="quote">COMPLEX SYSTEM STACK LEADS TO A SIMPLISTIC USER JOURNEY</div>
      <b-row class="justify-content-center mr-0 ml-0">
        <b-col cols="auto">
          <b-card
            title="Ensemble Learning"
            tag="article"
            style="max-width: 20rem"
            class="mb-2"
          >
            <b-card-text> 
             Sybl is encourages developers to construct their models in isolation and utilises these models to solve data science problems
            </b-card-text>
            <b-button href="#" variant="primary">Learn More</b-button>
          </b-card>
        </b-col>
        <b-col cols="auto">
          <b-card
            title="Data Privacy"
            tag="article"
            style="max-width: 20rem"
            class="mb-2"
          >
            <b-card-text>
              We have actively considered relevant data protection legislation during the design of the Sybl platform, empoying the CASTLEGUARD algorithm
            </b-card-text>
            <b-button href="#" variant="primary">Learn More</b-button>
          </b-card>
        </b-col>
        <b-col cols="auto">
          <b-card
            title="Failure Detection"
            tag="article"
            style="max-width: 20rem"
            class="mb-2"
          >
            <b-card-text>
              To ensure complete processing we have built a perfect failure detector. As such you can be asured a complete result without any loss of data
            </b-card-text>
            <b-button href="#" variant="primary">Learn More</b-button>
          </b-card>
        </b-col>
      </b-row>
    </div>
  </div>
</template>

<style>
.main {
  text-align: center;
  padding: 6rem 0;
}

.title {
  font-weight: bold;
  font-size: 3.5rem;
}

.subTitle {
  font-weight: normal;
  font-size: 2rem;
}

.quote {
  font-weight: bold;
  font-size: 1rem;
  padding-top: 2rem;
  padding-bottom: 1rem;
}

.fix-button {
  width: auto;
  margin: 1rem;
}

.outlined-fix-button {
  width: auto;
  margin: 1rem;
}

.highlight {
  padding-bottom: 2rem;
  text-align: center;
  background-color: #fcbf90;
  color: #a97600;
}
</style>

<script>
import PlaceHolder from "@/components/icons/PlaceHolder";
import { KerasIcon, TensorFlowIcon } from "vue-simple-icons";

export default {
  name: "Welcome",
  components: {
    placeholder: PlaceHolder,
    KerasIcon,
    TensorFlowIcon,
  },
};
</script>
