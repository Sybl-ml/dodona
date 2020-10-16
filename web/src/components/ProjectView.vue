<template>
  <b-container fluid>
    <h2>{{ name }}</h2>
    <h5>{{ description }}</h5>
    <b-tabs>
      <b-tab title="Overview" active>
        <br />This will be the Overview of the project
      </b-tab>
      <b-tab title="Input">
        <br />This will show the input and some basic graphs
      </b-tab>
      <b-tab title="Ouptut">
        <br />This will show the output from the machine learning methods
      </b-tab>
    </b-tabs>
  </b-container>
</template>

<script>
export default {
  name: "ProjectView",
  data() {
    return {};
  },
  props: ["project_id", "name", "description"],
};
</script>
