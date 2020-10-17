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
  border: none !important;
  border-bottom: 1px solid black !important;
  border-radius: 0 !important;
  font-size: x-large !important;
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
