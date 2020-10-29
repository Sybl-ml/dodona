<template>
  <b-container fluid>
    <h2>{{ name }}</h2>
    <h5>{{ description }}</h5>
    <p>{{ getProjectDate }}</p>
    <b-tabs>
      <b-tab title="Overview" active>
        <b-container fluid>
          <b-row>
            <b-col>
              <h4>&lt;Dataset Title&gt;</h4>
              <p>{{ getDatasetDate }}</p>
            </b-col>
          </b-row>
          <b-table striped :items="this.dataHead.data" />
          <b-row>
            <b-col class="text-center" style="color: #4650e8">
              <b-link @click="$refs.dataTab.activate()"
                >See More...</b-link
              ></b-col
            ></b-row
          >
        </b-container>
      </b-tab>
      <b-tab title="Input" ref="dataTab">
        <br />This will show the input and some basic graphs
      </b-tab>
      <b-tab title="Ouptut">
        <br />This will show the output from the machine learning methods
      </b-tab>
    </b-tabs>
  </b-container>
</template>

<script>
import axios from "axios";
import Papa from "papaparse";

export default {
  name: "ProjectView",
  data() {
    return {
      dataHead: "",
      dataDate: "",
      dataTypes: {},
    };
  },
  props: {
    projectId: String,
    name: String,
    description: String,
    dateCreated: Date,
  },
  async mounted() {
    let project_response = await axios.get(
      `http://localhost:3001/api/projects/p/${this.projectId}`
    );

    let project_details = project_response.data.details;

    this.dataHead = Papa.parse(project_details.head, { header: true });

    this.dataDate = new Date(project_details.date_created.$date);
    this.dataTypes = project_details.column_types;
  },
  methods: {
    async getProjectData() {
      let data_response = await axios.get(
        `http://localhost:3001/api/projects/p/${this.projectId}/data`
      );

      console.log(data_response.data);
    },
  },
  computed: {
    getProjectDate() {
      return `${this.dateCreated.toLocaleString("en-GB", {
        dateStyle: "short",
      })} - ${this.dateCreated.toLocaleString("en-GB", {
        timeStyle: "short",
      })}`;
    },
    getDatasetDate() {
      return `${this.dataDate.toLocaleString("en-GB", {
        dateStyle: "short",
      })} - ${this.dataDate.toLocaleString("en-GB", {
        timeStyle: "short",
      })}`;
    },
  },
};
</script>
