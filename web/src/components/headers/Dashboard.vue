<template>
  <b-navbar>
    <b-navbar-brand to="/">
      <icon-logo width="5em" height="3em" :show_text="true" />
    </b-navbar-brand>

    <b-navbar-nav class="ml-auto">
      <img src="http://placehold.it/18x18" class="img-circle">
      <b-nav-item-dropdown text="User 1" right>
          <b-dropdown-item href="#">Settings</b-dropdown-item>
          <b-dropdown-item href="#">Help</b-dropdown-item>
          <b-dropdown-item href="#">Logout</b-dropdown-item>
        </b-nav-item-dropdown>
    </b-navbar-nav>
  </b-navbar>
</template>
<style>

.img-circle {
  width: 2.5rem;
  height: 2.5rem;
}

</style>
<script>
import IconBase from '../IconBase'
import IconLogo from '../icons/IconLogo'
export default {
  name: "Header",
  components: {
    IconLogo
  }

};
</script>
