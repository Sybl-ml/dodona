<template>
  <div>
    <b-row class="pb-3">
      <b-col class="main" md="12" lg="6">
        <b-row class="justify-content-center">
          <b-col md="12" lg="8">
            <h1 class="title">
              Data Science. Simplified.
            </h1>
            <h3 class="subtitle">
              Get access to Machine Learning with a <b>ZERO</b> code solution.
            </h3>
            <h4 class="mb-5">
              Works with
              <a href="https://www.tensorflow.org/"><tensor-flow-icon /> </a>
              &
              <a href="https://keras.io/"><keras-icon /></a>
            </h4>
            <b-button class="mr-3" variant="primary" size="lg" to="/register">
              I Have Data 💾
            </b-button>
            <b-button class="ml-3" size="lg" variant="primary">
              I Have Models 📙
            </b-button>
          </b-col>
        </b-row>
      </b-col>
      <b-col
        lg="6"
        sm="0"
        class="d-none d-lg-block main justify-content-center"
      >
        <landing class="svg_styling" />
      </b-col>
    </b-row>

    <waves />
    <div class="highlight">
      <b-row class="justify-content-center">
        <h1 class="mt-4 subTitle">
          WHAT IS SYBL?
        </h1>
      </b-row>
      <b-row class="justify-content-center">
        <h4 style="width:55%" class="mb-3">
          Sybl is a Machine Leaning as a service (MLAAS) platform designed to
          make Data Science as accessible as possible, regardless of
          <b-icon icon="cpu" /> Technical Resources,
          <b-icon icon="code-slash" /> Educational Background,
          <b-icon icon="cash-stack" /> Financial Status or
          <b-icon icon="diagram3" /> Use Case.
        </h4>
      </b-row>
      <b-row class="justify-content-center mr-0 ml-0">
        <b-col cols="auto" class="detail_card">
          <b-card
            tag="article"
            style="max-width: 20rem"
            class="mb-2 detail_card bg-light border-0 background"
          >
            <b-icon font-scale="4" icon="cpu" />
            <br />
            <h3>
              Technical Resources
            </h3>
            <b-card-text>
              Sybl runs in the cloud using secure, private and anonymised data
              pipelines to safley get insights from your data.
            </b-card-text>
          </b-card>
        </b-col>
        <b-col cols="auto" class="detail_card">
          <b-card
            tag="article"
            style="max-width: 20rem"
            class="mb-2 detail_card bg-light border-0 background"
          >
            <b-icon font-scale="4" icon="code-slash" />
            <br />
            <h3>
              Educational Background
            </h3>
            <b-card-text>
              Sybl requires ZERO code or Data Science experience. Get the
              insights you need without any of the faff.
            </b-card-text>
          </b-card>
        </b-col>
      </b-row>
      <b-row class="justify-content-center m-3">
        <b-col cols="auto" class="detail_card">
          <b-card
            tag="article"
            style="max-width: 20rem"
            class="mb-2 detail_card bg-light border-0 background"
          >
            <b-icon font-scale="4" icon="cash-stack" />
            <br />
            <h3>
              Financial Status
            </h3>
            <b-card-text>
              Sybl uses a pay-as-you-go business model, so you only pay for what
              you use. No subscriptions, no nonsense.
            </b-card-text>
          </b-card>
        </b-col>
        <b-col cols="auto">
          <b-card
            tag="article"
            style="max-width: 20rem"
            class="mb-2 detail_card bg-light border-0 background"
          >
            <b-icon font-scale="4" icon="diagram3" />
            <br />
            <h3>
              Use Case
            </h3>
            <b-card-text>
              Whatever data you collect, Sybl uses cutting edge Machine Learning
              to ensure that you always get the best psosible predictions.
            </b-card-text>
          </b-card>
        </b-col>
      </b-row>
    </div>
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 100 1440 260">
      <linearGradient id="grad1" x1="0%" y1="0%" x2="100%" y2="0%">
        <stop offset="0%" stop-color="#fb8c38" />
        <stop offset="100%" stop-color="#fb8c38" />
      </linearGradient>
      <path
        fill="url(#grad1)"
        fill-opacity="1"
        d="M0,320L34.3,304C68.6,288,137,256,206,234.7C274.3,213,343,203,411,218.7C480,235,549,277,617,266.7C685.7,256,754,192,823,170.7C891.4,149,960,171,1029,181.3C1097.1,192,1166,192,1234,186.7C1302.9,181,1371,171,1406,165.3L1440,160L1440,0L1405.7,0C1371.4,0,1303,0,1234,0C1165.7,0,1097,0,1029,0C960,0,891,0,823,0C754.3,0,686,0,617,0C548.6,0,480,0,411,0C342.9,0,274,0,206,0C137.1,0,69,0,34,0L0,0Z"
      ></path>
    </svg>
    <b-container>
      <b-row class="justify-content-center shadow-none">
          <sybl width="24em" height="8em" />
      </b-row>
      <b-row class="m-4 justify-content-center shadow-none">
        <h1 class="subTitle">How Does It Work?</h1>
      </b-row>
      <b-row class="m-3 justify-content-center">
        <b-col>
          <b-row>
            <b-col sm="12" lg="6">
              <b-card class="text-center border-0 shadow-none">
                <h2>Upload Your Data</h2>
                <b-card-text
                  >If you have data that you want predictions on, upload it to
                  Sybl. It can be anything - financial forecasts, customer
                  insights, even sports results! All data you upload is
                  anonymused and made private before being used for Machine
                  Leaning.
                </b-card-text>
              </b-card>
            </b-col>

            <b-col sm="12" lg="6">
              <b-row class="justify-content-center">
                <cloud />
              </b-row>
            </b-col>
          </b-row>

          <b-row>
            <b-col sm="12" lg="6">
              <b-row class="justify-content-center">
                <terminal />
              </b-row>
            </b-col>
            <b-col sm="12" lg="6">
              <b-card class="text-center border-0 shadow-none">
                <h2>Create A Job</h2>
                <b-card-text
                  >Your anonymised data is sent to the world's top data
                  scientists, who use their top machine learning models to
                  analyse and make predictions on your data
                </b-card-text>
              </b-card>
            </b-col>
          </b-row>

          <b-row>
            <b-col sm="12" lg="6">
              <b-card class="text-center border-0 shadow-none">
                <h2>Get Your Jobs</h2>
                <b-card-text
                  >Your anonymised data is sent to the world's top data
                  scientists, who use their top machine learning models to
                  analyse and make predictions on your data
                </b-card-text>
              </b-card>
            </b-col>
            <b-col sm="12" lg="6">
              <b-row class="justify-content-center">
                <drop />
              </b-row>
            </b-col>
          </b-row>
        </b-col>
      </b-row>
    </b-container>
    <waves />

    <div class="highlight pt-3">
      <b-row class="justify-content-center">
        <h1 class="mt-3 subTitle">Get Started Today!</h1>
      </b-row>
      <b-row class="justify-content-center">
        <b-button class="mr-3" variant="ready" size="lg" to="/register">
          I Have Data 💾
        </b-button>
        <b-button class="ml-3" variant="ready" size="lg">
          I Have Models 📙
        </b-button>
      </b-row>
    </div>
  </div>
</template>

<style>
body {
  overflow-x: hidden !important;
}

.background {
  background-color: #ffffff;
  background-image: url("data:image/svg+xml,%3Csvg width='100' height='100' viewBox='0 0 100 100' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M11 18c3.866 0 7-3.134 7-7s-3.134-7-7-7-7 3.134-7 7 3.134 7 7 7zm48 25c3.866 0 7-3.134 7-7s-3.134-7-7-7-7 3.134-7 7 3.134 7 7 7zm-43-7c1.657 0 3-1.343 3-3s-1.343-3-3-3-3 1.343-3 3 1.343 3 3 3zm63 31c1.657 0 3-1.343 3-3s-1.343-3-3-3-3 1.343-3 3 1.343 3 3 3zM34 90c1.657 0 3-1.343 3-3s-1.343-3-3-3-3 1.343-3 3 1.343 3 3 3zm56-76c1.657 0 3-1.343 3-3s-1.343-3-3-3-3 1.343-3 3 1.343 3 3 3zM12 86c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm28-65c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm23-11c2.76 0 5-2.24 5-5s-2.24-5-5-5-5 2.24-5 5 2.24 5 5 5zm-6 60c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm29 22c2.76 0 5-2.24 5-5s-2.24-5-5-5-5 2.24-5 5 2.24 5 5 5zM32 63c2.76 0 5-2.24 5-5s-2.24-5-5-5-5 2.24-5 5 2.24 5 5 5zm57-13c2.76 0 5-2.24 5-5s-2.24-5-5-5-5 2.24-5 5 2.24 5 5 5zm-9-21c1.105 0 2-.895 2-2s-.895-2-2-2-2 .895-2 2 .895 2 2 2zM60 91c1.105 0 2-.895 2-2s-.895-2-2-2-2 .895-2 2 .895 2 2 2zM35 41c1.105 0 2-.895 2-2s-.895-2-2-2-2 .895-2 2 .895 2 2 2zM12 60c1.105 0 2-.895 2-2s-.895-2-2-2-2 .895-2 2 .895 2 2 2z' fill='%23fb8c38' fill-opacity='0.31' fill-rule='evenodd'/%3E%3C/svg%3E");
}
.main {
  text-align: center;
}

.title {
  padding-top: 8rem;
  font-weight: bold;
  font-size: 4rem;
}

.subTitle {
  font-weight: normal;
  font-size: 4rem;
}

.quote {
  font-weight: bold;
  font-size: 1rem;
  padding-top: 2rem;
  padding-bottom: 1rem;
}

.highlight {
  padding-bottom: 2rem;
  text-align: center;
  background-image: linear-gradient(to bottom, #fb8c38, #fbb000,  #fb8c38);
  color: white;
}

.detail_card {
  color: black;
  height: 17em;
}

.svg_styling {
  padding-top: 3rem;
  max-width: 30em;
}

.hero {
  width: 100%;
  height: auto;
}
</style>

<script>
import LandingImage from "@/components/icons/LandingImage";
import Waves from "@/components/icons/waves";
import { KerasIcon, TensorFlowIcon } from "vue-simple-icons";
import Cloud from "@/components/icons/cloud-1";
import Drop from "@/components/icons/drop";
import Box from "@/components/icons/box-1";
import Terminal from "@/components/icons/terminal";
import Sybl from "@/components/icons/IconLogo";

export default {
  name: "Welcome",
  components: {
    KerasIcon,
    TensorFlowIcon,
    landing: LandingImage,
    Waves,
    Cloud,
    Box,
    Drop,
    Terminal,
    Sybl,
  },
};
</script>
