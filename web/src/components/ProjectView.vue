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
          v-on:input-tab="$refs.dataTab.activate()"
        />
      </b-tab>
      <b-tab title="Input" ref="dataTab" lazy>
        <project-input :projectId="projectId" :key="projectId" />
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
  components: {
    ProjectOverview,
    ProjectInput,
  },
  methods: {},
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
