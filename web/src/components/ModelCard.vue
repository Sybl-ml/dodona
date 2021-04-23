<template>
  <b-container fluid>
    <b-row class="justify-content-center">
      <b-col xs="12" lg="7">
        <b-card
          class="mb-4 shadow"
          no-body
          v-b-toggle="`collapse-${this.i}`"
          :border-variant="status_variant"
          style="border-width: 0.15rem"
          onfocus="this.blur();"
        >
          <b-row no-gutter>
            <b-col>
              <b-card-body v-if="!model.locked">
                <b-card-title>
                  {{ model.name }}
                </b-card-title>
                <b-card-text v-if="model.status == 'Running'">
                  <b-icon-check-circle-fill
                    small
                    style="color: #00bf26"
                  ></b-icon-check-circle-fill>
                  Running
                </b-card-text>
                <b-card-text v-else-if="model.status == 'Stopped'">
                  <b-icon-x-octagon-fill
                    style="color: #ff643d"
                  ></b-icon-x-octagon-fill>
                  Stopped
                </b-card-text>
                <b-card-text v-else-if="model.status == 'NotStarted'">
                  <b-icon-pause-fill style="color: #fbb000"></b-icon-pause-fill>
                  Not Started
                </b-card-text>
                <b-card-text v-else>
                  <b-icon-exclamation-triangle-fill
                    style="color: #ff1700"
                  ></b-icon-exclamation-triangle-fill>
                  Error
                </b-card-text>
                <b-card-text>
                  <b-icon-clock-fill></b-icon-clock-fill>
                  {{ model.processing_time_secs }}s total processing
                </b-card-text>
                <b-card-text>
                  <b-icon-cash-stack></b-icon-cash-stack>
                  {{ model.credits_earned }} credit(s) earned
                </b-card-text>
              </b-card-body>
              <b-card-body v-else style="color: #7c7c7c">
                <b-card-title >
                  {{ model.name }}
                </b-card-title >
                  <b-card-text>
                    <b-icon-lock-fill style="color: #000000"></b-icon-lock-fill>
                    Locked
                  </b-card-text>
            </b-card-body>

            </b-col>
            <b-col>
              <b-card-body v-if="(model.status == 'Running' || model.status == 'Stopped') && this.loaded">
                <speedometer
                  :id="`spedometer-${i}`"
                  :performance="performance"
                />
                <b-tooltip :target="`spedometer-${i}`" variant="primary" placement="right" triggers="hover">
                  How well this model performed against other models 
                  over the last 5 jobs
                </b-tooltip>
              </b-card-body>
            </b-col>
          </b-row>
          <b-row no-gutter class="justify-content-center">
            <b-icon-chevron-compact-down
              font-scale="1.5"
            ></b-icon-chevron-compact-down>
          </b-row>
        </b-card>
      </b-col>
    </b-row>
    <b-row class="justify-content-center">
      <b-col xs="12" lg="7">
        <b-collapse
          :id="`collapse-${i}`"
          class="mb-4 nodeExpansion"
          @shown="renderChart(i)"
        >
          <b-card class="shadow" v-if="model.locked == true">
            <b-row class="justify-content-center text-center">
              <b-col md="8" sm="10" xs="12">
                <br />
                <h1>
                  <b-icon-lock-fill style="color: #fbb000"></b-icon-lock-fill>
                </h1>
                <b-card-title>Unlock Model</b-card-title>
                <b-card-text
                  >Please provide your password to confirm</b-card-text
                >
                <b-form class="mt-3 mb-3" @submit.prevent="onSubmit">
                  <b-form-input
                    type="password"
                    id="name"
                    class="mb-3"
                    placeholder="Password"
                    v-model="password"
                  ></b-form-input>
                  <b-button type="submit" variant="primary" class="mb-3">
                    Unlock
                  </b-button>
                </b-form>
              </b-col>
            </b-row>
          </b-card>
          <b-card class="shadow" v-else-if="model.status == 'NotStarted'">
            <b-card-title>Ready to Start</b-card-title>
            <b-card-text>Run your model script to begin!</b-card-text>
          </b-card>
          <b-card
            class="shadow-sm p-3 mb-5 bg-white rounded"
            style="border-width: 0.15rem"
            v-else
          >
            <model-performance
              :data="performance"
              :ref="`model-performance-${i}`"
            />

            <b-card-text>Total Runs: {{ model.times_run }}</b-card-text>
          </b-card>
        </b-collapse>
      </b-col>
    </b-row>
  </b-container>
</template>

<style>
.nodeExpansion {
  width: 100%;
}
</style>

<script>
import ModelPerformance from "@/components/ModelPerformance";
import Speedometer from "@/components/charts/Speedometer";

export default {
  name: "ModelCard",
  props: {
    model: Object,
    i: Number,
  },
  data() {
    return {
      password: "",
      loaded: false
    };
  },
  components: {
    ModelPerformance,
    Speedometer,
  },
  async created() {
    this.$store.dispatch("getModelPerformance", this.model._id.$oid).then(
      (result) => {
        this.loaded = true;
    });

  },
  computed: {
    status_variant() {
      if (this.model.locked === true) {
        return "dark";
      } else if (this.model.status === "NotStarted") {
        return "primary";
      } else if (this.model.status === "Running") {
        return "completed";
      } else if (this.model.status === "Stopped") {
        return "warning";
      } else {
        return "secondary";
      }
    },
    performance() {
      return this.$store.getters.getModelPerformance(this.model._id.$oid);
    },
  },
  methods: {
    async onSubmit() {
      this.$store.dispatch("unlockModel", {
        model_id: this.model._id.$oid,
        password: this.password,
      });
    },
    renderChart(model_id) {
      if (this.model.status === "Running")
        this.$refs[`model-performance-${model_id}`].show();
    },
  },
};
</script>
