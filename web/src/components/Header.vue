<template>
  <b-navbar>
    <b-navbar-brand to="/">
      <icon-logo width="5em" height="3em" :show_text="true" />
</b-navbar-brand>
    <b-navbar-nav>
      <b-nav-item>Product</b-nav-item>
      <b-nav-item>Meet the Team</b-nav-item>
      <b-nav-item>Pricing</b-nav-item>
    </b-navbar-nav>

    <b-navbar-nav class="ml-auto">
      <b-nav-form>
        <b-nav-item><router-link to="/login">Sign In</router-link></b-nav-item>
        <b-button variant="primary" to="/register">SIGN UP NOW</b-button>
      </b-nav-form>
    </b-navbar-nav>
  </b-navbar>
</template>

<script>
import IconBase from './IconBase'
import IconLogo from './icons/IconLogo'
export default {
  name: "Header",
  components: {
    IconLogo
  }

};
</script>
