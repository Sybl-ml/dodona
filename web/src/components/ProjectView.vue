<template>
  <b-container fluid>
    <h2>{{ name }}</h2>
    <h5>{{ description }}</h5>
    <p>{{ getProjectDate }}</p>
    <b-tabs>
      <b-tab title="Overview" active lazy>
        <project-overview
          :projectId="projectId"
          :key="projectId"
          :dataHead="dataHead"
          :dataDate="datasetDate"
          :dataTypes="dataTypes"
          v-on:input-tab="$refs.dataTab.activate()"
        />
      </b-tab>
      <b-tab title="Input" ref="dataTab">
        <project-input
          :projectId="projectId"
          :key="projectId"
          v-on:getData="getData"
          :data="data"
          :loading="loading"
        />
      </b-tab>
      <b-tab title="Ouptut" lazy>
        <br />This will show the output from the machine learning methods
      </b-tab>
    </b-tabs>
  </b-container>
</template>

<script>
import axios from "axios";
import Papa from "papaparse";
import ProjectOverview from "@/components/ProjectOverview";
import ProjectInput from "@/components/ProjectInput";

export default {
  name: "ProjectView",
  data() {
    return {
      name: "",
      description: "",
      dateCreated: new Date(),

      datasetDate: new Date(),
      dataHead: {},
      dataTypes: {},

      data: {},
      loading: true,
    };
  },
  props: {
    projectId: String,
  },
  components: {
    ProjectOverview,
    ProjectInput,
  },
  watch: {
    projectId: function () {
      this.fetchData();
    },
  },
  async mounted() {
    this.fetchData();
  },
  methods: {
    async fetchData() {
      let project_response = await axios.get(
        `http://localhost:3001/api/projects/p/${this.projectId}`
      );

      let project_details = project_response.data.details;
      let project_info = project_response.data.project;

      this.name = project_info.name;
      this.description = project_info.description;
      this.dateCreated = new Date(project_info.date_created.$date);

      this.dataHead = Papa.parse(project_details.head, { header: true });
      this.datasetDate = new Date(project_details.date_created.$date);
      this.dataTypes = project_details.column_types;
    },
    async getData() {
      console.log("data");
      let project_response = await axios.get(
        `http://localhost:3001/api/projects/p/${this.projectId}/data`
      );

      let project_data = project_response.data.dataset;

      this.data = Papa.parse(project_data, { header: true });
      this.loading = false;
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
  },
};
</script>
