<template>
  <b-container fluid>
    <b-form-input
      ref="title"
      placeholder="Name Your Project"
      class="add-project"
    />
    <b-form-input
      placeholder="Write a short description of your project"
      class="add-project"
    />
  </b-container>
</template>

<style>
.add-project {
  border: none;
  border-bottom: 1px solid black;
  border-radius: 0;
  font-size: 20px;
}
</style>

<script>
export default {
  name: "AddProject",
  data() {
    return {};
  },
  mounted() {
    this.$refs.title.focus();
  },
};
</script>
