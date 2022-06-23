import re
from turtle import title
from django.shortcuts import render, redirect
from .forms import EmailForm


def index(request):
	return render(request, 'main/index.html', {'title': 'Homepage'})


def lighsaber(request):
	return render(request, 'main/lightsaber.html', {'title': 'Lightsaber'})


def about(request):
	return render(request, 'main/about.html', {'title': 'About Us'})


def contacts(request):
	error = ''
	if request.method == 'EMAIL':
		email = EmailForm(request.EMAIL)
		if email.is_valid():
			email.save()
			return redirect('contacts')
		else:
			error = 'Form was incorrect'

	email = EmailForm

	data = {
	    'title': 'Contacts',
	    'email': email,
	    'error': error,
	}
	return render(request, 'main/contacts.html', data)