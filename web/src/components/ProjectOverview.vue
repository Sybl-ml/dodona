<template>
  <b-container fluid>
    <b-container v-if="ready">
      <h3>Almost Done!</h3>
      <p>To start computation click the button below</p>
      <br>
      <b-dropdown id="dropdown-form" text="Job Configuration" block variant="outline-primary" ref="dropdown" class="m-2" menu-class="w-100">
        <b-dropdown-text>This is where you configure how the job should be run</b-dropdown-text>
        <b-dropdown-form>
        <b-form-group label="Timeout (mins)" label-for="dropdown-form-timeout">
          <b-form-input
            id="dropdown-form-timeout"
            size="sm"
            type="number"
            v-model="timeout"
          ></b-form-input>
        </b-form-group>
        </b-dropdown-form>
      </b-dropdown>
      <br>
      <p class="display-1 text-center">
        <b-link @click="start">
          <b-icon-play-fill variant="success"/>
        </b-link>
      </p>
    </b-container>
    <b-container v-else>
      <b-row>
        <b-col>
          <h4>&lt;Dataset Title&gt;</h4>
          <p>{{ getDatasetDate }}</p>
        </b-col>
      </b-row>
      <b-row class="input-table">
        <b-table hover striped :items="this.dataHead.data" />
      </b-row>
      <b-row>
        <b-col class="text-center" style="color: #4650e8">
          <b-link @click="$emit('input-tab')">See More...</b-link></b-col
        ></b-row
      >
    </b-container>
  </b-container>
</template>

<style scoped>
.input-table {
  overflow-y: scroll;
}
</style>

<script>
import axios from "axios";
import Papa from "papaparse";

export default {
  name: "ProjectOverview",
  data() {
    return {
      timeout: 10,
    };
  },
  props: {
    projectId: String,
    dataDate: Date,
    dataHead: Object,
    dataTypes: Object,
    ready: Boolean,
  },
  computed: {
    getDatasetDate() {
      return `${this.dataDate.toLocaleString("en-GB", {
        dateStyle: "short",
      })} - ${this.dataDate.toLocaleString("en-GB", {
        timeStyle: "short",
      })}`;
    },
  },
  methods: {
    async start() {
      let user_id = $cookies.get("token");
      try {
        await axios.post(
          `http://localhost:3001/api/projects/p/${this.projectId}/process`,
          {
            timeout: this.timeout,
          }
        );
      } catch (err) {
        console.log(err);
      }
      
      // this.$router.replace("/dashboard/"+this.projectId);
      this.$emit("update:project", this.projectId);
    },
  }
};
</script>
